extends PanelContainer

signal show_model_info(model: Model)

## Set this before adding this node to the scene tree.
@export var model: Model = null

func _ready() -> void:
	if !model:
		return
	
	%Name.text = model.name
	%Name.tooltip_text = model.name
	
	model.cover_image_changed.connect(_update_cover_image)
	
	_update_cover_image()


func _update_cover_image():
	# Cover image can be null, but that simply means "no cover image"
	%CoverImage.icon = model.cover_image

func _on_open_button_pressed() -> void:
	Utils.open_with_default_program(model.directory)


func _on_cover_image_pressed() -> void:
	show_model_info.emit(model)
