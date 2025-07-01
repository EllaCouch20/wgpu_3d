use wgpu_3d::window::App;

fn main() {
    pollster::block_on(App::run());
}
