extends PanelContainer

## Set this before adding this node to the scene tree.
@export var model: Model = null

func _ready() -> void:
	if !model:
		return
	
	%Name.text = model.name
	
	if !model.cover_image.is_empty():
		var image = Image.new()
		var err := image.load(model.cover_image)
		
		if err == OK:
			var texture = ImageTexture.create_from_image(image)
			%CoverImage.texture = texture
