extends PanelContainer

signal closing()

var mesh_path := ""

@onready var model_render := %ModelRender


func show_3d_file(absolute_path: String):
	mesh_path = absolute_path
	
	var result = STLIO.Importer.LoadFromPath(mesh_path)
	if result is ArrayMesh:
		model_render.show_model(result)
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
	Utils.select_file_in_file_manager(mesh_path)


func _gui_input(event: InputEvent) -> void:
	if !visible:
		return
	
	if event.is_action("ui_back"):
		hide_3d_file()
		accept_event()
