extends PanelContainer

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
	button.text = file
	button.alignment = HORIZONTAL_ALIGNMENT_LEFT
	button.clip_text = true
	button.flat = true
	button.pressed.connect(_on_printable_clicked.bind(file))
	
	printables_list.add_child(button)

func _add_file_to_rest_list(file: String):
	var label := Label.new()
	label.text = file
	rest_list.add_child(label)

func _is_file_printable(file: String) -> bool:
	for extension in printable_file_extensions:
		if file.ends_with(extension):
			return true
	
	return false

func _on_printable_clicked(file: String):
	var full_path := "%s/%s" % [model.directory, file]
	
	var result = STLIO.Importer.LoadFromPath(full_path)
	if result is ArrayMesh:
		print("Yes")
