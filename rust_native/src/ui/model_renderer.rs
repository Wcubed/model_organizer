use godot::classes::{
    ArrayMesh, Camera3D, Input, InputEvent, InputEventMouseMotion, MeshInstance3D, Node3D,
};
use godot::global::MouseButton;
use godot::prelude::*;

const ZOOM_SPEED_FACTOR: f32 = 0.1;

#[derive(GodotClass)]
#[class(init, base=Node3D)]
struct ModelRenderer {
    base: Base<Node3D>,

    /// To be calculated when loading the mesh.
    min_zoom_distance: f32,
    /// To be calculated when loading the mesh.
    max_zoom_distance: f32,

    #[init(node = "%Model")]
    model: OnReady<Gd<MeshInstance3D>>,
    #[init(node = "%ModelAnchor")]
    model_anchor: OnReady<Gd<Node3D>>,
    #[init(node = "%Camera3D")]
    camera: OnReady<Gd<Camera3D>>,
    #[init(node = "%CameraAnchor")]
    camera_anchor: OnReady<Gd<Node3D>>,
    #[init(val = OnReady::manual())]
    camera_reset_rotation: OnReady<Vector3>,
}

#[godot_api]
impl INode3D for ModelRenderer {
    fn ready(&mut self) {
        // Remember the starting rotation.
        self.camera_reset_rotation
            .init(self.camera_anchor.get_rotation_degrees());
    }

    fn input(&mut self, event: Gd<InputEvent>) {
        if event.is_action("zoom_in") {
            self.zoom(-ZOOM_SPEED_FACTOR);
        } else if event.is_action("zoom_out") {
            self.zoom(ZOOM_SPEED_FACTOR);
        } else if Input::singleton().is_mouse_button_pressed(MouseButton::LEFT)
            && let Ok(mouse_motion) = event.try_cast::<InputEventMouseMotion>()
        {
            let motion = mouse_motion.get_screen_relative();

            let mut rotation = self.camera_anchor.get_rotation_degrees();
            rotation.y -= motion.x;
            // Rotate vertically, limited between looking straight up, and straight down.
            rotation.x = (rotation.x - motion.y).clamp(-90.0, 90.0);
            self.camera_anchor.set_rotation_degrees(rotation);
        }
    }
}

#[godot_api]
impl ModelRenderer {
    #[func]
    fn show_model(&mut self, mesh: Gd<ArrayMesh>, default_orientation: i32) {
        self.model.set_mesh(&mesh);

        // Models are almost never positioned at the origin.
        // So we need to move them to correct this.
        let aabb = self.model.get_aabb();
        self.model.set_position(-aabb.center());

        let rotation = if default_orientation == 0 {
            // Z up.
            Vector3 {
                x: -90.0,
                y: 0.0,
                z: 0.0,
            }
        } else {
            // Y up
            Vector3::ZERO
        };
        self.model_anchor.set_rotation_degrees(rotation);

        self.min_zoom_distance = aabb.shortest_axis_size() * 0.5;
        self.max_zoom_distance = aabb.longest_axis_size() * 2.0;

        // Reset the camera.
        self.camera.set_position(Vector3 {
            x: 0.0,
            y: 0.0,
            z: aabb.longest_axis_size(),
        });
        self.camera_anchor
            .set_rotation_degrees(*self.camera_reset_rotation);
    }

    #[func]
    fn remove_model(&mut self) {
        self.model.set_mesh(Gd::null_arg());
    }
}

impl ModelRenderer {
    fn zoom(&mut self, factor: f32) {
        let mut pos = self.camera.get_position();
        pos.z = (pos.z + pos.z * factor).clamp(self.min_zoom_distance, self.max_zoom_distance);
        self.camera.set_position(pos);
    }
}
