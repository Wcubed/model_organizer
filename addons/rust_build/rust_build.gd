@tool
extends EditorPlugin

func _enable_plugin() -> void:
	# Add autoloads here.
	pass


func _disable_plugin() -> void:
	# Remove autoloads here.
	pass


func _enter_tree() -> void:
	# Initialization of the plugin goes here.
	pass


func _exit_tree() -> void:
	# Clean-up of the plugin goes here.
	pass


func _build() -> bool:
	var output = []
	var result := OS.execute("./rust_build.sh", [], output, true)
	
	if result != 0:
		print("--- Cargo error ---\n")
		print(output[0])
		
		# Show the output panel.
		var bottom_panel := EditorInterface.get_base_control().find_child("*EditorBottomPanel*", true, false)
		var output_panel := bottom_panel.find_child("Output", false, false)
		var scroll_bar := output_panel.find_child("*VScrollBar*", true, false)
		output_panel.visible = true
		# Scroll to bottom.
		scroll_bar.value = scroll_bar.max_value
	
	return result == 0
