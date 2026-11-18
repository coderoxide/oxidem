use std::collections::HashMap;
use crate::hir::*;
use crate::span::Span;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoanKind {
    Shared,
    Mutable,
}

#[derive(Debug, Clone)]
pub struct ActiveLoan {
    pub base_var: String,
    pub ref_var: String,
    pub kind: LoanKind,
    pub span: Span,
}

pub struct BorrowChecker {
    // Current active loans: ref_var -> ActiveLoan
    active_loans: HashMap<String, ActiveLoan>,
}

impl BorrowChecker {
    pub fn new() -> Self {
        Self {
            active_loans: HashMap::new(),
        }
    }

    pub fn check_program(hir_prog: &HirProgram) -> Result<(), String> {
        for func in &hir_prog.functions {
            let mut checker = BorrowChecker::new();
            checker.check_function(func)?;
        }
        Ok(())
    }

    pub fn check_function(&mut self, func: &HirFunction) -> Result<(), String> {
        // Step 1: Compute liveness (last use point) for each variable
        let last_uses = Self::compute_last_uses(func);

        // Step 2: Linear CFG statement verification
        for (bb_idx, bb) in func.blocks.iter().enumerate() {
            for (stmt_idx, stmt) in bb.statements.iter().enumerate() {
                let loc = (bb_idx, stmt_idx);

                // Expire loans whose reference variable's last use was BEFORE this location
                self.expire_dead_loans(&loc, &last_uses);

                match stmt {
                    HirStatement::Assign { place, rvalue, span } => {
                        // Check if rvalue takes a new borrow
                        match rvalue {
                            Rvalue::Ref { is_mut, place: base_place } => {
                                if let Place::Local(base_var) = base_place {
                                    self.check_can_borrow(base_var, *is_mut, *span)?;
                                    if let Place::Local(ref_var) = place {
                                        let kind = if *is_mut {
                                            LoanKind::Mutable
                                        } else {
                                            LoanKind::Shared
                                        };
                                        self.active_loans.insert(
                                            ref_var.clone(),
                                            ActiveLoan {
                                                base_var: base_var.clone(),
                                                ref_var: ref_var.clone(),
                                                kind,
                                                span: *span,
                                            },
                                        );
                                    }
                                }
                            }
                            Rvalue::Use(Operand::Copy(Place::Local(src_var))) => {
                                // Variable alias tracking / loan propagation (e.g. r = _t0)
                                if let Place::Local(dest_var) = place {
                                    if let Some(loan) = self.active_loans.get(src_var).cloned() {
                                        let mut propagated = loan;
                                        propagated.ref_var = dest_var.clone();
                                        self.active_loans.insert(dest_var.clone(), propagated);
                                    }
                                }
                                // Check read access on src_var
                                self.check_can_read(src_var, *span)?;
                            }
                            Rvalue::BinaryOp(_, left, right) => {
                                if let Operand::Copy(Place::Local(v)) = left {
                                    self.check_can_read(v, *span)?;
                                }
                                if let Operand::Copy(Place::Local(v)) = right {
                                    self.check_can_read(v, *span)?;
                                }
                            }
                            Rvalue::UnaryOp(_, op) => {
                                if let Operand::Copy(Place::Local(v)) = op {
                                    self.check_can_read(v, *span)?;
                                }
                            }
                            _ => {}
                        }

                        // Check if this assignment mutates a borrowed variable
                        if let Place::Local(target_var) = place {
                            // If place is not creating a reference, check if target_var is currently borrowed
                            if !matches!(rvalue, Rvalue::Ref { .. }) {
                                self.check_can_mutate(target_var, *span)?;
                            }
                        }
                    }
                    HirStatement::Call { args, span, .. } => {
                        for a in args {
                            if let Operand::Copy(Place::Local(v)) = a {
                                self.check_can_read(v, *span)?;
                            }
                        }
                    }
                }
            }

            // At terminator, check operands
            match &bb.terminator {
                Terminator::Branch { cond, span, .. } => {
                    if let Operand::Copy(Place::Local(v)) = cond {
                        self.check_can_read(v, *span)?;
                    }
                }
                Terminator::Return(Some(Operand::Copy(Place::Local(v))), span) => {
                    self.check_can_read(v, *span)?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn check_can_borrow(&self, base_var: &str, is_mut: bool, span: Span) -> Result<(), String> {
        for loan in self.active_loans.values() {
            if loan.base_var == base_var {
                match loan.kind {
                    LoanKind::Shared => {
                        if is_mut {
                            return Err(format!(
                                "Borrow Hatası [{}:{}]: '{}' değişkeni zaten paylaşılmış (&) olarak ödünç alınmışken değiştirilebilir (&mut) olarak ödünç alınamaz (İlk ödünç: satır {})",
                                span.line, span.col, base_var, loan.span.line
                            ));
                        }
                    }
                    LoanKind::Mutable => {
                        return Err(format!(
                            "Borrow Hatası [{}:{}]: '{}' değişkeni zaten &mut olarak ödünç alınmışken ikinci bir referans alınamaz (Yalnızca tek bir &mut bulunabilir)",
                            span.line, span.col, base_var
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn check_can_mutate(&self, target_var: &str, span: Span) -> Result<(), String> {
        for loan in self.active_loans.values() {
            if loan.base_var == target_var {
                match loan.kind {
                    LoanKind::Shared => {
                        return Err(format!(
                            "Borrow Hatası [{}:{}]: '{}' değişkeni paylaşılmış (&) referansla ödünç alınmışken yeniden atanamaz / değiştirilemez",
                            span.line, span.col, target_var
                        ));
                    }
                    LoanKind::Mutable => {
                        return Err(format!(
                            "Borrow Hatası [{}:{}]: '{}' değişkeni değiştirilebilir (&mut) ödünç alınmışken doğrudan değiştirilemez",
                            span.line, span.col, target_var
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn check_can_read(&self, var_name: &str, span: Span) -> Result<(), String> {
        for loan in self.active_loans.values() {
            if loan.base_var == var_name && loan.kind == LoanKind::Mutable {
                // If the variable being read is the ref itself, that's allowed!
                if loan.ref_var != var_name {
                    return Err(format!(
                        "Borrow Hatası [{}:{}]: '{}' değişkeni değiştirilebilir (&mut) olarak ödünç alınmışken doğrudan okunamaz",
                        span.line, span.col, var_name
                    ));
                }
            }
        }
        Ok(())
    }

    fn expire_dead_loans(&mut self, current_loc: &(usize, usize), last_uses: &HashMap<String, (usize, usize)>) {
        let mut expired = Vec::new();
        for (ref_var, _) in &self.active_loans {
            if let Some(last_loc) = last_uses.get(ref_var) {
                // Non-Lexical Lifetimes: If current location is past the last use, expire the loan!
                if current_loc > last_loc {
                    expired.push(ref_var.clone());
                }
            } else {
                // If ref_var has no uses in instructions, it expires
                expired.push(ref_var.clone());
            }
        }
        for ref_var in expired {
            self.active_loans.remove(&ref_var);
        }
    }

    /// Computes the last instruction (bb_idx, stmt_idx) where each variable is used (read or dereferenced)
    fn compute_last_uses(func: &HirFunction) -> HashMap<String, (usize, usize)> {
        let mut last_uses = HashMap::new();

        for (bb_idx, bb) in func.blocks.iter().enumerate() {
            for (stmt_idx, stmt) in bb.statements.iter().enumerate() {
                let loc = (bb_idx, stmt_idx);
                match stmt {
                    HirStatement::Assign { rvalue, .. } => match rvalue {
                        Rvalue::Use(Operand::Copy(Place::Local(v))) => {
                            last_uses.insert(v.clone(), loc);
                        }
                        Rvalue::BinaryOp(_, left, right) => {
                            if let Operand::Copy(Place::Local(v)) = left {
                                last_uses.insert(v.clone(), loc);
                            }
                            if let Operand::Copy(Place::Local(v)) = right {
                                last_uses.insert(v.clone(), loc);
                            }
                        }
                        Rvalue::UnaryOp(_, op) => {
                            if let Operand::Copy(Place::Local(v)) = op {
                                last_uses.insert(v.clone(), loc);
                            }
                        }
                        Rvalue::Array(elems) => {
                            for e in elems {
                                if let Operand::Copy(Place::Local(v)) = e {
                                    last_uses.insert(v.clone(), loc);
                                }
                            }
                        }
                        Rvalue::Struct(_, fields) => {
                            for (_, f_op) in fields {
                                if let Operand::Copy(Place::Local(v)) = f_op {
                                    last_uses.insert(v.clone(), loc);
                                }
                            }
                        }
                        _ => {}
                    },
                    HirStatement::Call { args, .. } => {
                        for a in args {
                            if let Operand::Copy(Place::Local(v)) = a {
                                last_uses.insert(v.clone(), loc);
                            }
                        }
                    }
                }
            }
        }

        last_uses
    }
}
