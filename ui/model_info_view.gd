extends PanelContainer

## User requests that the given 3d file be shown in the main viewer.
signal show_3d_file(absolute_path: String, default_orientation: Utils.ModelOrientation)
signal render_icon_for_3d_file(absolute_path: String, model: Model)

var model: Model = null

@onready var printables_list := %PrintablesList
@onready var rest_list := %RestList
@onready var rendering_status_label := %RenderingStatusLabel
@onready var model_orientation_options := %ModelOrientationOptions
@onready var printable_entry_scene := preload("res://ui/printable_entry_card.tscn")

func _ready() -> void:
	rendering_status_label.text = ""

## Display the given model. The search string is used to display which
## printables match that search.
func display_model(new_model: Model, search_string: String):
	clear_model()
	
	model = new_model
	
	%NameLabel.text = model.name
	%ModelOrientationOptions.select(model.default_orientation)
	%CoverImage.texture = model.cover_image
	
	for file in model.printable_files:
		# Is there a pre-rendered image for this one?
		var rendered_path := Utils.strip_extension(file) + ".png"
		if model.rendered_files.find(rendered_path) == -1:
			# No rendered image yet. Queue the render.
			_queue_render_icon(file)
			rendered_path = ""
		
		_add_file_to_printables(file, rendered_path, search_string)
	for file in model.misc_files:
		_add_file_to_rest_list(file)

func clear_model():
	model = null
	%NameLabel.text = ""
	%CoverImage.texture = null
	%PrintablesScrollContainer.scroll_vertical = 0
	%RestScrollContainer.scroll_vertical = 0
	
	for child in printables_list.get_children():
		printables_list.remove_child(child)
		child.queue_free()
	
	for child in rest_list.get_children():
		rest_list.remove_child(child)
		child.queue_free()

func _queue_render_icon(file: String):
	render_icon_for_3d_file.emit("%s/%s" % [model.directory, file], model)


func clear_printable_selection():
	for child in printables_list.get_children():
		child.show_selected(false)


## An image has been rendered in the background, check if one needs to be updated.
func background_render_done(absolute_image_path: String, texture: ImageTexture):
	if !absolute_image_path.begins_with(model.directory):
		# This is not an image that belongs to our model.
		return
	
	for child in printables_list.get_children():
		child.background_render_done(absolute_image_path, texture)

func render_queue_length_changed(new_length: int):
	if new_length == 0:
		rendering_status_label.text = ""
	else:
		rendering_status_label.text = "Rendering %d models..." % new_length


func render_all_model_previews():
	for file in model.printable_files:
		_queue_render_icon(file)


## Rendered path may be empty, signyfing that there is no render yet.
func _add_file_to_printables(file: String, rendered_path: String, search_string: String):
	var entry := printable_entry_scene.instantiate()
	printables_list.add_child(entry)
	
	var is_search_result := Utils.string_matches_search_pattern(file, search_string) == Utils.StringMatchResult.SUCCESS
	
	entry.display_file(model.directory, file, rendered_path, is_search_result)
	entry.preview_printable.connect(_on_printable_clicked)

func _add_file_to_rest_list(file: String):
	var label := Label.new()
	label.text = file.trim_prefix("/")
	rest_list.add_child(label)

func _on_printable_clicked(absolute_file: String, control: Control):
	clear_printable_selection()
	control.show_selected(true)
	
	show_3d_file.emit(absolute_file, model.default_orientation)


func _on_rerender_button_pressed() -> void:
	render_all_model_previews()


func _on_model_orientation_options_item_selected(index: int) -> void:
	# User changed the prefered orientation of the model.
	model.default_orientation = index as Utils.ModelOrientation
	render_all_model_previews()
