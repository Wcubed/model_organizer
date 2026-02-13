extends HBoxContainer

signal preview_printable(file: String, control: Control)

var file: String = ""


# Pass the absolute file path.
func display_file(new_file: String):
	file = new_file
	# Show only the filename, with the full path in the tooltip.
	%NameButton.text = file.split("/")[-1]
	%NameButton.tooltip_text = file.trim_prefix("/")


func show_selected(selected: bool):
	if selected:
		%NameButton.add_theme_color_override("font_color", Color.GREEN)
	else:
		%NameButton.remove_theme_color_override("font_color")


func _on_external_button_pressed() -> void:
	OS.shell_open(file)


func _on_folderbutton_pressed() -> void:
	var last_slash := file.rfind("/")
	var folder_path = file.substr(0, last_slash)
	OS.shell_open(folder_path)


func _on_name_button_pressed() -> void:
	preview_printable.emit(file, self)
