extends PanelContainer

## Set this before adding this node to the scene tree.
@export var model: Model = null

func _ready() -> void:
	if !model:
		return
	
	%Name.text = model.name
	
	if model.cover_image != null:
		%CoverImage.texture = model.cover_image


func _on_open_button_pressed() -> void:
	OS.shell_open(model.directory)
