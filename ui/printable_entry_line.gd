extends HBoxContainer

signal preview_printable(file: String, control: Control)

var file: String = ""

@onready var name_button := %NameButton

## Call this after adding the entry to the list.
## Pass the absolute file path.
func display_file(new_file: String):
	file = new_file
	# Show only the filename, with the full path in the tooltip.
	name_button.text = file.split("/")[-1]
	name_button.tooltip_text = file.trim_prefix("/")


func show_selected(selected: bool):
	if selected:
		name_button.add_theme_color_override("font_color", Color.GREEN)
		name_button.add_theme_color_override("font_hover_color", Color.LIGHT_GREEN)
	else:
		name_button.remove_theme_color_override("font_color")
		name_button.remove_theme_color_override("font_hover_color")


func _on_external_button_pressed() -> void:
	OS.shell_open(file)


func _on_folderbutton_pressed() -> void:
	var last_slash := file.rfind("/")
	var folder_path = file.substr(0, last_slash)
	OS.shell_open(folder_path)


func _on_name_button_pressed() -> void:
	preview_printable.emit(file, self)
