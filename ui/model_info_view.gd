extends PanelContainer

var model: Model = null

@onready var files_list := %FilesList

func display_model(new_model: Model):
	model = new_model
	
	%NameLabel.text = model.name
	if model.cover_image != null:
		%CoverImage.icon = model.cover_image
	
	for child in files_list.get_children():
		files_list.remove_child(child)
		child.queue_free()
	
	for file in model.files:
		var label := Label.new()
		label.text = file
		files_list.add_child(label)
