extends PanelContainer

signal closing()

var mesh_path := ""

@onready var model_render := %ModelRender


func show_3d_file(absolute_path: String, default_orientation: Utils.ModelOrientation):
	mesh_path = absolute_path
	
	var result = ResourceLoader.load(mesh_path)
	if result is ArrayMesh:
		print(result.get_surface_count())
		model_render.show_model(result, default_orientation)
	else:
		model_render.remove_model()
	
	show()

func hide_3d_file():
	model_render.remove_model()
	hide()
	closing.emit()

func _on_close_button_pressed() -> void:
	hide_3d_file()


func _on_open_external_button_pressed() -> void:
	Utils.open_with_default_program(mesh_path)


func _gui_input(event: InputEvent) -> void:
	if !visible:
		return
	
	if event.is_action("ui_back"):
		hide_3d_file()
		accept_event()
