extends PanelContainer

signal preview_printable(file: String, control: Control)

var file: String = ""

@onready var name_label := %NameLabel
@onready var rendered_image := %RenderedImage

## Call this after adding the entry to the scene tree.
## The `maybe_rendered_path` may be empty, which means there is no render yet.
func display_file(model_base_dir: String, new_file: String, maybe_rendered_path: String):
	file = "%s/%s" % [model_base_dir, new_file]
	# Show only the filename, with the full path in the tooltip.
	name_label.text = file.split("/")[-1]
	name_label.tooltip_text = file.trim_prefix("/")
	
	# Attempt to load the rendered image.
	if !maybe_rendered_path.is_empty():
		var image := Image.new()
		var result := image.load("%s/%s" % [model_base_dir, maybe_rendered_path])
		if result == OK:
			var texture := ImageTexture.create_from_image(image)
			rendered_image.icon = texture


func show_selected(selected: bool):
	if selected:
		name_label.add_theme_color_override("font_color", Color.GREEN)
		name_label.add_theme_color_override("font_hover_color", Color.LIGHT_GREEN)
	else:
		name_label.remove_theme_color_override("font_color")
		name_label.remove_theme_color_override("font_hover_color")

## An image has been rendered in the background, check if we need to update.
func background_render_done(absolute_image_path: String, texture: ImageTexture):
	if Utils.strip_extension(file) == Utils.strip_extension(absolute_image_path):
		# This is our texture.
		rendered_image.icon = texture

func _on_external_button_pressed() -> void:
	Utils.open_with_default_program(file)


func _on_folderbutton_pressed() -> void:
	Utils.select_file_in_file_manager(file)

func _on_rendered_image_pressed() -> void:
	preview_printable.emit(file, self)
