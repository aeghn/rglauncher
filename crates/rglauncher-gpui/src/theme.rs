use gpui::*;

#[allow(dead_code)]
pub struct Theme {
    pub text: Rgba,
    pub subtext1: Rgba,
    pub subtext0: Rgba,
    pub overlay2: Rgba,
    pub overlay1: Rgba,
    pub overlay0: Rgba,
    pub surface2: Rgba,
    pub surface1: Rgba,
    pub surface0: Rgba,
    pub base: Rgba,
    pub base_blur: Rgba,
    pub mantle: Rgba,
    pub crust: Rgba,
    pub crust_light: Rgba,
}

impl Global for Theme {}

impl Theme {
    pub fn init(cx: &mut WindowContext) {
        let theme = Theme::get_dark();
        cx.set_global(theme);
    }

    // Catppuccin Mocha
    // Text	#cdd6f4	rgb(205, 214, 244)	hsl(226, 64%, 88%)
    // Subtext1	#bac2de	rgb(186, 194, 222)	hsl(227, 35%, 80%)
    // Subtext0	#a6adc8	rgb(166, 173, 200)	hsl(228, 24%, 72%)
    // Overlay2	#9399b2	rgb(147, 153, 178)	hsl(228, 17%, 64%)
    // Overlay1	#7f849c	rgb(127, 132, 156)	hsl(230, 13%, 55%)
    // Overlay0	#6c7086	rgb(108, 112, 134)	hsl(231, 11%, 47%)
    // Surface2	#585b70	rgb(88, 91, 112)	hsl(233, 12%, 39%)
    // Surface1	#45475a	rgb(69, 71, 90)	hsl(234, 13%, 31%)
    // Surface0	#313244	rgb(49, 50, 68)	hsl(237, 16%, 23%)
    // Base	#1e1e2e	rgb(30, 30, 46)	hsl(240, 21%, 15%)
    // Mantle	#181825	rgb(24, 24, 37)	hsl(240, 21%, 12%)
    // Crust	#11111b	rgb(17, 17, 27)	hsl(240, 23%, 9%)
    pub fn get_dark() -> Theme {
        Theme {
            text: rgb(0xcdd6f4),
            subtext1: rgb(0xbac2de),
            subtext0: rgb(0xa6adc8),
            overlay2: rgb(0x9399b2),
            overlay1: rgb(0x7f849c),
            overlay0: rgb(0x6c7086),
            surface2: rgb(0x585b70),
            surface1: rgb(0x45475a),
            surface0: rgb(0x313244),
            base: rgb(0x1e1e2e),
            base_blur: rgba(0x1e1e2edd),
            mantle: rgb(0x181825),
            crust: rgb(0x11111b),
            crust_light: rgba(0x6c708666),
        }
    }
}
