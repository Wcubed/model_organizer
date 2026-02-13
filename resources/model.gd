class_name Model
extends Resource

## A single model can contain multiple stl files.

@export var directory: String
@export var name: String
## List of files in the directory, without the base path.
@export var files: Array[String]
## Absolute path to the cover image.
@export var cover_image: String = ""

var supported_image_extensions := [".jpg", ".jpeg", ".png", ".webp"]

func _init(p_directory = ""):
	directory = p_directory
	name = directory.split("/")[-1].split(".")[0]

## Scans the model directory for relevant info.
## Call this once after creating the resource.
func scan_directory():
	files = []
	_scan_subdirectory(directory)
	
	find_cover_image()

func find_cover_image():
	# See if we can find a cover image.
	for file in files:
		for extension in supported_image_extensions:
			if file.ends_with(extension):
				cover_image = "%s/%s" % [directory, file]
				return
	
	# No cover image.
	cover_image = ""

func _scan_subdirectory(path: String):
	var dir = DirAccess.open(path)
	if !dir:
		return
	
	# Directory exists. Scan it.
	var new_files = Array(dir.get_files())
	files.append_array(new_files)
	
	# Scan subdirectories
	var subdirs = dir.get_directories()
	for subdir in subdirs:
		var new_path = "%s/%s" % [path, subdir]
		_scan_subdirectory(new_path)
