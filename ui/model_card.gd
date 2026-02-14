extends PanelContainer

signal show_model_info(model: Model)

## Set this before adding this node to the scene tree.
@export var model: Model = null

func _ready() -> void:
	if !model:
		return
	
	%Name.text = model.name
	%Name.tooltip_text = model.name
	
	if model.cover_image != null:
		%CoverImage.icon = model.cover_image


func _on_open_button_pressed() -> void:
	OS.shell_open(model.directory)


func _on_cover_image_pressed() -> void:
	show_model_info.emit(model)
