@tool
extends EditorPlugin

var build_icon := preload("res://addons/rust_build/BuildPlay.svg")

var export_plugin: RustExportPlugin = null
var run_buttons_hbox: HBoxContainer = null
var build_button: Button = null
## This is the default "run main scene" button
## It is hidden while the plugin is active.
var run_button: Button = null

func _enable_plugin() -> void:
	pass


func _disable_plugin() -> void:
	pass


func _enter_tree() -> void:
	# Initialization of the plugin goes here.
	export_plugin = RustExportPlugin.new(get_tree())
	add_export_plugin(export_plugin)
	
	# Find the "Run main scene" button.
	var buttons: Array[Node] = get_editor_interface().get_base_control().find_child("*EditorRunBar*", true, false).find_children("*", "Button", true, false)
	
	for item in buttons:
		if item is Button:
			if item.tooltip_text == "Run the project's main scene.":
				# Found the button.
				run_button = item
				
				run_buttons_hbox = item.get_parent()
				break
	
	run_button.visible = false
	
	build_button = Button.new()
	build_button.icon = build_icon
	build_button.flat = true
	build_button.tooltip_text = "Build and run"
	build_button.pressed.connect(_on_build_button_pressed)
	
	run_buttons_hbox.add_child(build_button)
	run_buttons_hbox.move_child(build_button, 0)


func _exit_tree() -> void:
	# Clean-up of the plugin goes here.
	remove_export_plugin(export_plugin)
	
	run_buttons_hbox.remove_child(build_button)
	run_button.visible = true


func _on_build_button_pressed():
	export_plugin.start_build_command("./rust_build_linux.sh")


func _process(delta: float) -> void:
	if export_plugin.is_build_running():
		if not export_plugin.update_running_build():
			# Build complete.
			if export_plugin.exit_code == 0:
				# Build success! Start the main scene.
				get_editor_interface().play_main_scene()
