use crate::user_interface::ui_element::{UIElementVisitor, Button, Slider, UIText, UIElementTrait};
use crate::user_interface::text_render::TextRenderer;

pub struct TextRenderVisitor<'a> {
    pub text_renderer: &'a TextRenderer,
}

impl<'a> UIElementVisitor for TextRenderVisitor<'a> {
    fn visit_button(&mut self, _button: &mut Button, _is_clicked: bool) {}
    fn visit_slider(&mut self, _slider: &mut Slider) {}
    fn visit_text(&mut self, text: &mut UIText) {
        print!("visiting text");
        self.text_renderer.render_text(
            text.get_text(),
            text.get_position().x + 1.0, // adjust as needed
            text.get_position().y + 1.0 + text.get_font_size(), // adjust as needed
            1.0,
            cgmath::Vector3::new(0.3, 0.4, 1.0), // black
            &cgmath::ortho(0.0, 720.0, 720.0, 0.0, -1.0, 1.0),//todo this needs to be dynamic and not ass like bruh i am just stetting width and height to 720 exactly and hoping it works basically
        );
    }
}