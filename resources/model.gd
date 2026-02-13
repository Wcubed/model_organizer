class_name Model
extends Resource

## A single model can contain multiple stl files.

@export var directory: String
@export var name: String

func _init(p_directory = ""):
	directory = p_directory
	name = directory.split("/")[-1].split(".")[0]


## Scans the model directory for relevant info.
func scan_directory():
	pass
