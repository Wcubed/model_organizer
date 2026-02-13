extends PanelContainer

## Set this before adding this node to the scene tree.
@export var model: Model = null

func _ready() -> void:
	if !model:
		return
	
	%Name.text = model.name
