extends PanelContainer

signal preview_printable(file: String, control: Control)

var file: String = ""

@onready var name_label := %NameLabel
@onready var rendered_image := %RenderedImage
@onready var normal_panel := preload("res://ui/assets/card_panel.tres")
@onready var highlight_panel := preload("res://ui/assets/card_panel_highlight.tres")

## Call this after adding the entry to the scene tree.
## The `maybe_rendered_path` may be empty, which means there is no render yet.
func display_file(model_base_dir: String, new_file: String, maybe_rendered_path: String, is_search_result: bool):
	file = "%s/%s" % [model_base_dir, new_file]
	# Show only the filename
	var filename := file.split("/")[-1]
	name_label.text = filename
	name_label.tooltip_text = filename
	
	if is_search_result:
		name_label.add_theme_color_override("font_color", Color.GREEN)
		name_label.add_theme_color_override("font_hover_color", Color.LIGHT_GREEN)
	
	# Attempt to load the rendered image.
	if !maybe_rendered_path.is_empty():
		var image := Image.new()
		var result := image.load("%s/%s" % [model_base_dir, maybe_rendered_path])
		if result == OK:
			var texture := ImageTexture.create_from_image(image)
			rendered_image.icon = texture


func show_selected(selected: bool):
	if selected:
		add_theme_stylebox_override("panel", highlight_panel)
	else:
		add_theme_stylebox_override("panel", normal_panel)

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
