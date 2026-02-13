extends PanelContainer

signal show_array_mesh(mesh: ArrayMesh, path: String)

var model: Model = null

## These extensions can be put on the 3d printer. Others can't.
var printable_file_extensions := [".stl", ".3mf"]

@onready var printables_list := %PrintablesList
@onready var rest_list := %RestList

func display_model(new_model: Model):
	model = new_model
	
	%NameLabel.text = model.name
	if model.cover_image != null:
		%CoverImage.texture = model.cover_image
	
	for child in printables_list.get_children():
		printables_list.remove_child(child)
		child.queue_free()
	
	for child in rest_list.get_children():
		rest_list.remove_child(child)
		child.queue_free()
	
	for file in model.files:
		if _is_file_printable(file):
			_add_file_to_printables(file)
		else:
			_add_file_to_rest_list(file)

func _add_file_to_printables(file: String):
	var button := Button.new()
	# Show only the filename, with the full path in the tooltip.
	button.text = file.split("/")[-1]
	button.tooltip_text = file.trim_prefix("/")
	button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	button.flat = true
	button.mouse_default_cursor_shape = Control.CURSOR_POINTING_HAND
	button.pressed.connect(_on_printable_clicked.bind(file))
	
	printables_list.add_child(button)

func _add_file_to_rest_list(file: String):
	var label := Label.new()
	label.text = file.trim_prefix("/")
	rest_list.add_child(label)

func _is_file_printable(file: String) -> bool:
	for extension in printable_file_extensions:
		if file.to_lower().ends_with(extension):
			return true
	
	return false

func _on_printable_clicked(file: String):
	var full_path := "%s/%s" % [model.directory, file]
	
	var result = STLIO.Importer.LoadFromPath(full_path)
	if result is ArrayMesh:
		show_array_mesh.emit(result, full_path)
