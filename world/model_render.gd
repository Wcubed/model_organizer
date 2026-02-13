extends Node3D

@onready var model := %Model
@onready var camera := %Camera3D

func show_model(mesh: ArrayMesh):
	model.mesh = mesh
	
	# Models are almost never positioned at the origin.
	# So we need to move them to correct this.
	var aabb: AABB = model.get_aabb()
	model.position = -aabb.get_center()
	
	camera.position.z = aabb.get_longest_axis_size()
