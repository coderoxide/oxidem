// Oxide SDL2 Oyun Motoru Entegrasyon Testi

extern "C" {
    fn SDL_Init(flags: int) -> int;
    fn SDL_Quit() -> void;
}

fn main() {
    println("=== Oxide SDL2 Oyun Motoru Entegrasyon Testi ===");
    unsafe {
        // SDL_INIT_VIDEO = 32
        let res = SDL_Init(32);
        if res == 0 {
            println("✔ SDL2 Video Altsistemi başarıyla başlatıldı!");
            SDL_Quit();
            println("✔ SDL2 başarıyla kapatıldı.");
        } else {
            println("SDL2 Init durumu:", res);
        }
    }
    println("=== SDL2 Testi Tamamlandı ===");
}
