extends PanelContainer

signal show_3d_file(absolute_path: String)

var model: Model = null

## These extensions can be put on the 3d printer. Others can't.
var printable_file_extensions := [".stl", ".3mf"]

@onready var printables_list := %PrintablesList
@onready var rest_list := %RestList
@onready var printable_entry_scene := preload("res://ui/printable_entry_line.tscn")

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

func clear_printable_selection():
	for child in printables_list.get_children():
		child.show_selected(false)

func _add_file_to_printables(file: String):
	var entry := printable_entry_scene.instantiate()
	printables_list.add_child(entry)
	
	entry.display_file("%s/%s" % [model.directory, file])
	entry.preview_printable.connect(_on_printable_clicked)

func _add_file_to_rest_list(file: String):
	var label := Label.new()
	label.text = file.trim_prefix("/")
	rest_list.add_child(label)

func _is_file_printable(file: String) -> bool:
	for extension in printable_file_extensions:
		if file.to_lower().ends_with(extension):
			return true
	
	return false

func _on_printable_clicked(absolute_file: String, control: Control):
	clear_printable_selection()
	control.show_selected(true)
	
	show_3d_file.emit(absolute_file)
