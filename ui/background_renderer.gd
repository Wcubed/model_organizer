extends Node

## Queue of files to render.
var render_queue: Array[String] = []
## If this is not empty, it means the viewport rendered the given file this frame
## and we can now retrieve the result
var viewport_rendered_once := ""

@onready var viewport := %SubViewport
@onready var model_renderer := %ModelRender

func _ready() -> void:
	viewport.size.x = Utils.ICON_SIZE
	viewport.size.y = Utils.ICON_SIZE


func _process(_delta: float) -> void:
	if !viewport_rendered_once.is_empty():
		# A render is done.
		var rendered_texture: ViewportTexture = viewport.get_texture()
		var image := rendered_texture.get_image()
		
		var image_path := Utils.strip_extension(viewport_rendered_once) + ".png"
		
		image.save_png(image_path)
		
		viewport_rendered_once = ""
	
	if render_queue.is_empty():
		return
	
	var next_file := render_queue[0]
	var status := ResourceLoader.load_threaded_get_status(next_file)
	
	if status == ResourceLoader.THREAD_LOAD_INVALID_RESOURCE:
		# Not tried to load this yet.
		_start_next_file_load()
	elif status == ResourceLoader.THREAD_LOAD_FAILED:
		# Something is wrong with this file, we cannot load it.
		# Continue to the next file.
		render_queue.pop_front()
		_start_next_file_load()
	elif status == ResourceLoader.THREAD_LOAD_LOADED:
		# Load done. Do the render.
		render_queue.pop_front()
		_start_next_file_load()
		
		var mesh: ArrayMesh = ResourceLoader.load_threaded_get(next_file)
		
		model_renderer.show_model(mesh)
		viewport.render_target_update_mode = SubViewport.UPDATE_ONCE
		viewport_rendered_once = next_file
	else:
		# Nothing to do, wait another round.
		pass


## Starts loading the first item in the queue.
func _start_next_file_load():
	if render_queue.is_empty():
		return
	
	var next_file := render_queue[0]
	ResourceLoader.load_threaded_request(next_file)


func add_icon_to_queue(model_path: String):
	render_queue.append(model_path)
