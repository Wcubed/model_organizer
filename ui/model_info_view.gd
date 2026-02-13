extends PanelContainer

signal show_array_mesh(mesh: ArrayMesh, path: String)

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

func _add_file_to_printables(file: String):
	var entry := printable_entry_scene.instantiate()
	entry.display_file("%s/%s" % [model.directory, file])
	entry.preview_printable.connect(_on_printable_clicked)
	
	printables_list.add_child(entry)

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
	for child in printables_list.get_children():
		child.show_selected(false)
	
	var result = STLIO.Importer.LoadFromPath(absolute_file)
	if result is ArrayMesh:
		show_array_mesh.emit(result, absolute_file)
		control.show_selected(true)
