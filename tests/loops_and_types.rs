// Integration tests for Oxide
use std::process::Command;

#[test]
fn test_typecheck_and_run_for_loops() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/for_loops.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/for_loops.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());
}

#[test]
fn test_build_and_execute_native() {
    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/for_loops.ox", "-o", "examples/test_native_for"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_for")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    // Cleanup
    let _ = std::fs::remove_file("examples/test_native_for");
    let _ = std::fs::remove_file("examples/test_native_for.c");
}

#[test]
fn test_break_outside_loop_error() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/break_outside_loop_error.ox"])
        .output()
        .expect("cargo run check failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("yalnızca bir döngü içerisinde kullanılabilir"));
}

#[test]
fn test_modules_and_imports() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/modules_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/modules_demo.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/modules_demo.ox", "-o", "examples/test_native_modules"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_modules")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_modules");
    let _ = std::fs::remove_file("examples/test_native_modules.c");
}

#[test]
fn test_diamond_import() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/diamond_main.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());
}

#[test]
fn test_circular_import_error() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/circ_a.ox"])
        .output()
        .expect("cargo run check failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Döngüsel içe aktarma"));
}

#[test]
fn test_missing_import_error() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/missing_import_error.ox"])
        .output()
        .expect("cargo run check failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("bulunamadı"));
}

#[test]
fn test_c_ffi_math() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/ffi_math.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/ffi_math.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/ffi_math.ox", "-o", "examples/test_native_math"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_math")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_math");
    let _ = std::fs::remove_file("examples/test_native_math.c");
}

#[test]
fn test_file_io() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/file_io.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/file_io.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/file_io.ox", "-o", "examples/test_native_io"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_io")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_io");
    let _ = std::fs::remove_file("examples/test_native_io.c");
    let _ = std::fs::remove_file("examples/test_io.txt");
}

#[test]
fn test_chars_and_bytes() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/chars_and_bytes.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/chars_and_bytes.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/chars_and_bytes.ox", "-o", "examples/test_native_chars"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_chars")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_chars");
    let _ = std::fs::remove_file("examples/test_native_chars.c");
}

#[test]
fn test_hir_generation() {
    let hir_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "hir", "examples/chars_and_bytes.ox"])
        .status()
        .expect("cargo run hir failed");
    assert!(hir_status.success());
}

#[test]
fn test_borrow_valid_and_nll() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/borrow_valid.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/borrow_valid.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/borrow_valid.ox", "-o", "examples/test_native_borrow"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_borrow")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_borrow");
    let _ = std::fs::remove_file("examples/test_native_borrow.c");
}

#[test]
fn test_borrow_error_mutate_while_borrowed() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/borrow_error_mutate_while_borrowed.ox"])
        .output()
        .expect("cargo run check failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Borrow Hatası") && stderr.contains("ödünç alınmışken yeniden atanamaz"));
}

#[test]
fn test_borrow_error_mut_and_shared() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/borrow_error_mut_and_shared.ox"])
        .output()
        .expect("cargo run check failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Borrow Hatası") && stderr.contains("değiştirilebilir (&mut) olarak ödünç alınamaz"));
}

#[test]
fn test_borrow_error_immutable_mut_borrow() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/borrow_error_immutable_mut_borrow.ox"])
        .output()
        .expect("cargo run check failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Borrow Hatası") && stderr.contains("Değişmez (immutable)"));
}

#[test]
fn test_unsafe_error_extern_outside() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/unsafe_error_extern_outside.ox"])
        .output()
        .expect("cargo run check failed");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Güvenlik Hatası") && stderr.contains("'unsafe' blok içinde yapılmalıdır"));
}

#[test]
fn test_stdlib_methods_and_system() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/stdlib_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/stdlib_demo.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/stdlib_demo.ox", "-o", "examples/test_native_stdlib"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_stdlib")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_stdlib");
    let _ = std::fs::remove_file("examples/test_native_stdlib.c");
}

#[test]
fn test_hir_optimizer() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "hir", "examples/opt_demo.ox", "--opt"])
        .output()
        .expect("cargo run hir --opt failed");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Verified: folded constants in basic block
    assert!(stdout.contains("_t0 = 40"));
    assert!(stdout.contains("_t1 = 50"));
    assert!(stdout.contains("b = 35"));
    // Verified: dead basic block 999 eliminated
    assert!(!stdout.contains("999"));
}

#[test]
fn test_oxide_test_runner() {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "test", "examples/test_suite.ox"])
        .output()
        .expect("cargo run test failed");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("4 geçti; 0 başarısız oldu"));
}

#[test]
fn test_project_init_and_manifest() {
    let tmp_proj = "examples/test_auto_project";
    let _ = std::fs::remove_dir_all(tmp_proj);

    let new_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "new", tmp_proj])
        .status()
        .expect("cargo run new failed");
    assert!(new_status.success());

    assert!(std::path::Path::new(tmp_proj).join("oxide.toml").exists());
    assert!(std::path::Path::new(tmp_proj).join("src/main.ox").exists());
    assert!(std::path::Path::new(tmp_proj).join("tests/test_basic.ox").exists());

    // Test runner inside the new project
    let test_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "test", tmp_proj])
        .status()
        .expect("cargo run test on new project failed");
    assert!(test_status.success());

    // Cleanup
    let _ = std::fs::remove_dir_all(tmp_proj);
}

#[test]
fn test_closures_and_functional_methods() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/closures_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/closures_demo.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/closures_demo.ox", "-o", "examples/test_native_closures"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_closures")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_closures");
    let _ = std::fs::remove_file("examples/test_native_closures.c");
}

#[test]
fn test_generics_and_parametric_polymorphism() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/generics_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/generics_demo.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/generics_demo.ox", "-o", "examples/test_native_generics"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_generics")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_generics");
    let _ = std::fs::remove_file("examples/test_native_generics.c");
}

#[test]
fn test_error_handling_option_result_try() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/error_handling_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/error_handling_demo.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/error_handling_demo.ox", "-o", "examples/test_native_error_handling"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_error_handling")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_error_handling");
    let _ = std::fs::remove_file("examples/test_native_error_handling.c");
}

#[test]
fn test_traits_and_interfaces() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/traits_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/traits_demo.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/traits_demo.ox", "-o", "examples/test_native_traits"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_traits")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_traits");
    let _ = std::fs::remove_file("examples/test_native_traits.c");
}

#[test]
fn test_tuples_and_multi_return() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/tuples_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/tuples_demo.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/tuples_demo.ox", "-o", "examples/test_native_tuples"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_tuples")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_tuples");
    let _ = std::fs::remove_file("examples/test_native_tuples.c");
}

#[test]
fn test_string_interpolation_and_format() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/format_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/format_demo.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/format_demo.ox", "-o", "examples/test_native_format"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_format")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_format");
    let _ = std::fs::remove_file("examples/test_native_format.c");
}

#[test]
fn test_slices_and_range_indexing() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/slices_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/slices_demo.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/slices_demo.ox", "-o", "examples/test_native_slices"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_slices")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_slices");
    let _ = std::fs::remove_file("examples/test_native_slices.c");
}

#[test]
fn test_operator_overloading() {
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/operator_overload_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    let run_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/operator_overload_demo.ox"])
        .status()
        .expect("cargo run run failed");
    assert!(run_status.success());

    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/operator_overload_demo.ox", "-o", "examples/test_native_ops"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    let exec_status = Command::new("./examples/test_native_ops")
        .status()
        .expect("native binary execution failed");
    assert!(exec_status.success());

    let _ = std::fs::remove_file("examples/test_native_ops");
    let _ = std::fs::remove_file("examples/test_native_ops.c");
}

#[test]
fn test_project_target_lock_and_clean() {
    let test_dir = "tests/temp_test_project";
    let _ = std::fs::remove_dir_all(test_dir);

    // 1. oxide new
    let new_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "new", test_dir])
        .status()
        .expect("cargo run new failed");
    assert!(new_status.success());

    assert!(std::path::Path::new(&format!("{}/oxide.toml", test_dir)).exists());
    assert!(std::path::Path::new(&format!("{}/oxide.lock", test_dir)).exists());
    assert!(std::path::Path::new(&format!("{}/.gitignore", test_dir)).exists());
    assert!(std::path::Path::new(&format!("{}/src/main.ox", test_dir)).exists());

    let gitignore = std::fs::read_to_string(format!("{}/.gitignore", test_dir)).unwrap();
    assert!(gitignore.contains("/target/"));

    let lockfile = std::fs::read_to_string(format!("{}/oxide.lock", test_dir)).unwrap();
    assert!(lockfile.contains("[[package]]"));

    // 2. oxide build inside project
    let build_status = Command::new("cargo")
        .current_dir(test_dir)
        .args(["run", "--manifest-path", "../../Cargo.toml", "--quiet", "--", "build"])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    // Target binary and C file must be generated under target/debug
    let proj_name = "temp_test_project";
    let bin_path = format!("{}/target/debug/{}", test_dir, proj_name);
    let c_path = format!("{}/target/debug/c_gen/{}.c", test_dir, proj_name);
    assert!(std::path::Path::new(&bin_path).exists(), "Binary not found at {}", bin_path);
    assert!(std::path::Path::new(&c_path).exists(), "C file not found at {}", c_path);

    // 3. Execute built binary
    let exec_output = Command::new(&bin_path)
        .output()
        .expect("Failed to run built binary");
    assert!(exec_output.status.success());
    let stdout = String::from_utf8_lossy(&exec_output.stdout);
    assert!(stdout.contains("Merhaba, Oxide!"));

    // 4. oxide clean inside project
    let clean_status = Command::new("cargo")
        .current_dir(test_dir)
        .args(["run", "--manifest-path", "../../Cargo.toml", "--quiet", "--", "clean"])
        .status()
        .expect("cargo run clean failed");
    assert!(clean_status.success());

    assert!(!std::path::Path::new(&format!("{}/target", test_dir)).exists(), "target/ directory was not removed by clean");

    // Cleanup temp dir
    let _ = std::fs::remove_dir_all(test_dir);
}

#[test]
fn test_smart_pointers_and_recursive_structs() {
    // 1. check
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/smart_pointers_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    // 2. run (interpreter)
    let run_output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/smart_pointers_demo.ox"])
        .output()
        .expect("cargo run run failed");
    assert!(run_output.status.success());
    let run_stdout = String::from_utf8_lossy(&run_output.stdout);

    // 3. build (GCC native)
    let bin_path = "examples/test_native_smart_ptr";
    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/smart_pointers_demo.ox", "-o", bin_path])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    // 4. execute native binary
    let exec_output = Command::new(format!("./{}", bin_path))
        .output()
        .expect("failed to execute native binary");
    assert!(exec_output.status.success());
    let exec_stdout = String::from_utf8_lossy(&exec_output.stdout);

    // 5. Parity check & content assertion
    assert_eq!(run_stdout.trim(), exec_stdout.trim());
    assert!(run_stdout.contains("Box degeri (*b): 42"));
    assert!(run_stdout.contains("Bagli liste: 10 -> 20 -> 30 -> Son"));
    assert!(run_stdout.contains("Liste elemanlari toplami: 60"));
    assert!(run_stdout.contains("rc1 baslangic referans sayisi: 1"));
    assert!(run_stdout.contains("rc2 klonlandi, guncel sayac: 2"));
    assert!(run_stdout.contains("cell.borrow_mut() ile guncellendi: 999"));

    // Cleanup
    let _ = std::fs::remove_file(bin_path);
    let _ = std::fs::remove_file(format!("{}.c", bin_path));
}

#[test]
fn test_const_and_filesystem() {
    // 1. check
    let check_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "check", "examples/const_and_fs_demo.ox"])
        .status()
        .expect("cargo run check failed");
    assert!(check_status.success());

    // 2. run (interpreter)
    let run_output = Command::new("cargo")
        .args(["run", "--quiet", "--", "run", "examples/const_and_fs_demo.ox"])
        .output()
        .expect("cargo run run failed");
    assert!(run_output.status.success());
    let run_stdout = String::from_utf8_lossy(&run_output.stdout);

    // 3. build (GCC native)
    let bin_path = "examples/test_native_const_fs_bin";
    let build_status = Command::new("cargo")
        .args(["run", "--quiet", "--", "build", "examples/const_and_fs_demo.ox", "-o", bin_path])
        .status()
        .expect("cargo run build failed");
    assert!(build_status.success());

    // 4. execute native binary
    let exec_output = Command::new(format!("./{}", bin_path))
        .output()
        .expect("failed to execute native binary");
    assert!(exec_output.status.success());
    let exec_stdout = String::from_utf8_lossy(&exec_output.stdout);

    // 5. Parity & content assertion
    assert_eq!(run_stdout.trim(), exec_stdout.trim());
    assert!(run_stdout.contains("BUFFER_SIZE (1024 * 4): 4096"));
    assert!(run_stdout.contains("GREETING: OxideFS Sistem Araci"));
    assert!(run_stdout.contains("BUFFER_SIZE / CARPAN (4096 / 12): 341"));
    assert!(run_stdout.contains("Dosya yazma: Basarili"));
    assert!(run_stdout.contains("Dosya mevcut mu?: true"));
    assert!(run_stdout.contains("Okunan icerik: Merhaba! Oxide guvenli sistem programlama dili dosya sistemi testi."));
    assert!(run_stdout.contains("Dosya silme: Basarili"));
    assert!(run_stdout.contains("Silme sonrasi mevcut mu?: false"));
    assert!(run_stdout.contains("Argumanlar basariyla alindi: true"));
    assert!(run_stdout.contains("PATH cevre degiskeni: Mevcut (is_some: true)"));

    // Cleanup
    let _ = std::fs::remove_file(bin_path);
    let _ = std::fs::remove_file(format!("{}.c", bin_path));
}


