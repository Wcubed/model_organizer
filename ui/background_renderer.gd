extends Node

signal render_done(absolute_image_path: String, texture: ImageTexture)
signal queue_length_changed(length: int)

## Queue of files to render.
## Contains [printable_path, model it belongs to]
var render_queue: Array[Array] = []
## If this is not empty, it means the viewport rendered the given file this frame
## and we can now retrieve the result
var viewport_rendered_once := []

@onready var viewport := %SubViewport
@onready var model_renderer := %ModelRender

func _ready() -> void:
	viewport.size.x = Utils.ICON_SIZE
	viewport.size.y = Utils.ICON_SIZE


func _process(_delta: float) -> void:
	if !viewport_rendered_once.is_empty():
		# A render is done.
		var viewport_texture: ViewportTexture = viewport.get_texture()
		var image := viewport_texture.get_image()
		var image_texture := ImageTexture.create_from_image(image)
		
		var image_path := Utils.strip_extension(viewport_rendered_once[0]) + ".png"
		render_done.emit(image_path, image_texture)
		
		# Save the newly generated image.
		var model: Model = viewport_rendered_once[1]
		var relative_image_path = image_path.trim_prefix(model.directory + "/")
		image.save_png(image_path)
		
		# We notify the model _after_ we have saved the image,
		# because the model might want to switch its preview because of
		# this change.
		model.new_rendered_file(relative_image_path)
		
		viewport_rendered_once = []
		model_renderer.remove_model()
	
	if render_queue.is_empty():
		return
	
	var next_item := render_queue[0]
	var status := ResourceLoader.load_threaded_get_status(next_item[0])
	
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
		
		var mesh: ArrayMesh = ResourceLoader.load_threaded_get(next_item[0])
		
		model_renderer.show_model(mesh, next_item[1].default_orientation)
		viewport.render_target_update_mode = SubViewport.UPDATE_ONCE
		viewport_rendered_once = next_item
	else:
		# Nothing to do, wait another round.
		pass


## Starts loading the first item in the queue.
func _start_next_file_load():
	queue_length_changed.emit(render_queue.size())
	
	if render_queue.is_empty():
		return
	
	var next_file := render_queue[0]
	ResourceLoader.load_threaded_request(next_file[0])


func add_icon_to_queue(printable_path: String, model: Model):
	var item := [printable_path, model]
	if render_queue.find(item) == -1:
		render_queue.append(item)


## Clears the rendering process. Use this when reloading the library to prevent broken references.
func stop_and_clear_queue():
	render_queue.clear()
	viewport_rendered_once = []
	model_renderer.remove_model()
	queue_length_changed.emit(render_queue.size())
