extends Node3D

# Z is 'up' in most models, so the camera angle is a bit weird for godot.
var anchor_start_angle := Vector3(45, -45, -45)

@onready var model := %Model
@onready var camera := %Camera3D
@onready var camera_anchor := %CameraAnchor

func show_model(mesh: ArrayMesh):
	model.mesh = mesh
	
	# Models are almost never positioned at the origin.
	# So we need to move them to correct this.
	var aabb: AABB = model.get_aabb()
	model.position = -aabb.get_center()
	
	camera.position.z = aabb.get_longest_axis_size()
	camera_anchor.rotation_degrees = anchor_start_angle
