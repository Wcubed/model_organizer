extends Node3D

## This one is going to be calculated when loading the mesh.
var min_zoom_distance := 1.0
## This one is going to be calculated when loading the mesh.
var max_zoom_distance := 2.0

var zoom_speed_factor := 0.1

@onready var model := %Model
@onready var camera := %Camera3D
@onready var camera_anchor := %CameraAnchor
## Z is 'up' in most models, so the camera angle is a bit weird for godots standard.
@onready var anchor_start_angle: Vector3 = camera_anchor.rotation_degrees

func show_model(mesh: ArrayMesh):
	model.mesh = mesh
	
	# Models are almost never positioned at the origin.
	# So we need to move them to correct this.
	var aabb: AABB = model.get_aabb()
	model.position = -aabb.get_center()
	
	min_zoom_distance = aabb.get_shortest_axis_size() * 0.5
	max_zoom_distance = aabb.get_longest_axis_size() * 2
	
	camera.position.z = aabb.get_longest_axis_size()
	camera_anchor.rotation_degrees = anchor_start_angle


func _input(event: InputEvent) -> void:
	if event.is_action("zoom_in"):
		camera.position.z -= camera.position.z * zoom_speed_factor
		camera.position.z = max(camera.position.z, min_zoom_distance)
	elif event.is_action("zoom_out"):
		camera.position.z += camera.position.z * zoom_speed_factor
		camera.position.z = min(camera.position.z, max_zoom_distance)
	elif event is InputEventMouseMotion && Input.is_mouse_button_pressed(MouseButton.MOUSE_BUTTON_LEFT):
		camera_anchor.rotation_degrees.z += event.screen_relative.x
		camera_anchor.rotation_degrees.x = max(min(camera_anchor.rotation_degrees.x + event.screen_relative.y, 90), 0)
		
		print(camera_anchor.rotation_degrees)
