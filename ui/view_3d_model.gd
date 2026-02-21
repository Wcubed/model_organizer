extends PanelContainer

signal closing()

var mesh_path := ""
var mesh_default_orientation := Utils.ModelOrientation.Z_UP

@onready var model_render := %ModelRender
@onready var loading_panel := %LoadingPanel
@onready var loading_show_timer := %LoadingShowTimer
@onready var failed_panel := %FailedPanel

func _process(_delta: float) -> void:	
	var status := ResourceLoader.load_threaded_get_status(mesh_path)
	
	if status == ResourceLoader.THREAD_LOAD_FAILED:
		# Something is wrong with this file, we cannot load it.
		model_render.remove_model()
		loading_panel.hide()
		loading_show_timer.stop()
		failed_panel.show()
	elif status == ResourceLoader.THREAD_LOAD_LOADED:
		# Load done
		var mesh: ArrayMesh = ResourceLoader.load_threaded_get(mesh_path)
		
		model_render.show_model(mesh, mesh_default_orientation)
		loading_panel.hide()
		loading_show_timer.stop()
	else:
		# Nothing to do.
		pass


func show_3d_file(absolute_path: String, default_orientation: Utils.ModelOrientation):
	# It is annoying to have the loading panel flicker in and out
	# if you already had a 3d model visible, in that case we make it appear only after the timer runs out.
	if mesh_path.is_empty():
		loading_panel.show()
	else:
		loading_show_timer.start()
	
	failed_panel.hide()
	
	mesh_path = absolute_path
	mesh_default_orientation = default_orientation
	
	ResourceLoader.load_threaded_request(mesh_path)
	
	show()

func hide_3d_file():
	mesh_path = ""
	model_render.remove_model()
	hide()
	closing.emit()

func _on_close_button_pressed() -> void:
	hide_3d_file()


func _on_open_external_button_pressed() -> void:
	Utils.open_with_default_program(mesh_path)


func _gui_input(event: InputEvent) -> void:
	if !visible:
		return
	
	if event.is_action("ui_back"):
		hide_3d_file()
		accept_event()


func _on_loading_show_timer_timeout() -> void:
	loading_panel.show()
