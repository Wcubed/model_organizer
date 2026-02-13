extends PanelContainer

signal close_button_pressed()

var mesh_path := ""

@onready var model_render := %ModelRender


func show_mesh(mesh: ArrayMesh, path: String):
	model_render.show_model(mesh)
	mesh_path = path
	
	self.show()

func hide_mesh():
	# TODO (2026-02-13): Unload the stl.
	self.hide()

func _on_close_button_pressed() -> void:
	hide_mesh()
	close_button_pressed.emit()


func _on_open_external_button_pressed() -> void:
	OS.shell_open(mesh_path)


func _gui_input(event: InputEvent) -> void:
	if event.is_action("ui_back"):
		hide_mesh()
