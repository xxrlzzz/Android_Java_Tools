pub trait RenderSource {
  fn render_file_info(&self) -> Vec<String>;
  fn render_class_info(&self) -> Vec<String>;
  fn render_interfaces(&self) -> Vec<String>;
  fn render_fields(&self) -> Vec<String>;
  fn render_methods(&self) -> Vec<String>;
  fn render_attributes(&self) -> Vec<String>;
  fn render_constant_pool(&self) -> Vec<String>;
}
