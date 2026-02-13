extends PanelContainer

signal close_button_pressed()

@onready var model_render := %ModelRender


func show_mesh(mesh: ArrayMesh):
	model_render.show_model(mesh)
	
	self.show()

func _on_close_button_pressed() -> void:
	# TODO (2026-02-13): Unload the stl.
	self.hide()
	close_button_pressed.emit()
