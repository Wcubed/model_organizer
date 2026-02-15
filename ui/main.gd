extends PanelContainer

@onready var amount_label := %AmountLabel
@onready var search_edit := %SearchEdit
@onready var folder_dialog := $FolderDialog
@onready var model_cards := %ModelCards
@onready var model_info_view := %ModelInfoView
@onready var view_3d_model := %View3dModel
@onready var search_edit_debounce := %SearchEditDebounce
@onready var background_rendered := %BackgroundRenderer
@onready var library_scanner := %LibraryScanner

var model_card_scene := preload("res://ui/model_card.tscn")

var config_file_name := "user://config.cfg"
var library_dir: String = ""

## All the models found in the library.
var models: Array[Model] = []
## All the models matching the search query.
var searched_models: Array[Model] = []

func _ready() -> void:
	# Make sure that the cache dir is created.
	DirAccess.make_dir_recursive_absolute(Utils.cover_image_cache_dir)
	
	load_settings()
	scan_library()

func save_settings():
	var config = ConfigFile.new()
	config.set_value("main", "library_dir", library_dir)
	
	config.save(config_file_name)


func load_settings():
	var config = ConfigFile.new()
	var err = config.load(config_file_name)
	if err != OK:
		return
	
	library_dir = config.get_value("main", "library_dir", "")


func scan_library():
	library_scanner.background_scan_library(library_dir)
	
	for card in model_cards.get_children():
		model_cards.remove_child(card)
		card.queue_free()
	view_3d_model.hide_3d_file()
	model_info_view.clear_model()


func _on_library_scan_complete(found_models: Array[Model]):
	models = found_models
	run_search_and_display()

## Searches the models.
## Clears the model cards and displays the currently found models.
func run_search_and_display():
	var search_text: String = search_edit.text
	
	if search_text.is_empty():
		searched_models = models
	else:
		searched_models = []
		for model in models:
			if model.matches_search(search_text):
				searched_models.append(model)
	
	# Display
	amount_label.text = "%d/%d" % [searched_models.size(), models.size()]
	
	for card in model_cards.get_children():
		model_cards.remove_child(card)
		card.queue_free()
	
	for model in searched_models:
		var new_card = model_card_scene.instantiate()
		new_card.model = model
		new_card.show_model_info.connect(_user_requests_show_model_info)
		model_cards.add_child(new_card)

func clear_and_select_search():
	var previous_text = search_edit.text
	search_edit.text = ""
	search_edit_debounce.stop()
	
	# No need to change the display if the search hasn't changed.
	if previous_text != search_edit.text:
		run_search_and_display()
	
	search_edit.grab_focus()


func _user_requests_show_model_info(model: Model):
	model_info_view.display_model(model)


func _on_path_button_pressed() -> void:
	folder_dialog.show()


func _on_folder_dialog_dir_selected(dir: String) -> void:
	library_dir = dir
	folder_dialog.current_dir = dir
	
	save_settings()
	scan_library()


func _on_reload_button_pressed() -> void:
	scan_library()


func _on_search_edit_text_changed(_new_text: String) -> void:
	# When the debounce timer runs out without being restarted, the actual search is performed.
	search_edit_debounce.start()

func _on_search_edit_text_submitted(_new_text: String) -> void:
	search_edit_debounce.stop()
	run_search_and_display()


func _on_clear_search_button_pressed() -> void:
	view_3d_model.hide_3d_file()
	clear_and_select_search()


func _on_model_info_view_show_3d_file(absolute_path: String) -> void:
	view_3d_model.show_3d_file(absolute_path)


func _on_search_edit_debounce_timeout() -> void:
	run_search_and_display()

func _on_search_edit_focus_entered() -> void:
	view_3d_model.hide_3d_file()


func _gui_input(event: InputEvent) -> void:
	if event.is_action("ui_back"):
		clear_and_select_search()
		accept_event()


func _on_view_3d_model_closing() -> void:
	model_info_view.clear_printable_selection()


func _on_model_info_view_render_icon_for_3d_file(absolute_path: String, model: Model) -> void:
	background_rendered.add_icon_to_queue(absolute_path, model)


func _on_background_renderer_render_done(absolute_image_path: String, texture: ImageTexture) -> void:
	model_info_view.background_render_done(absolute_image_path, texture)


func _on_background_renderer_queue_length_changed(length: int) -> void:
	model_info_view.render_queue_length_changed(length)
