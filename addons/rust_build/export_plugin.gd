extends EditorExportPlugin
class_name RustExportPlugin

func _get_name() -> String:
	return "RustExportPlugin"

func _export_begin(features: PackedStringArray, is_debug: bool, path: String, flags: int) -> void:
	if features.find("linux") != -1:
		build_rust_linux()
	if features.find("windows") != -1:
		build_rust_windows()


func build_rust_linux() -> bool:
	var output = []
	var result := OS.execute("./rust_build_linux.sh", [], output, true)
	
	if result != 0:
		output_cargo_error(output[0])
	
	return result == 0


func build_rust_windows() -> bool:
	var output = []
	var result := OS.execute("./rust_build_windows.sh", [], output, true)
	
	if result != 0:
		output_cargo_error(output[0])
	
	return result == 0


func output_cargo_error(error: String):
	push_error("Cargo build error:")
	print(error)
	
	# Show the output panel.
	var bottom_panel := EditorInterface.get_base_control().find_child("*EditorBottomPanel*", true, false)
	var output_panel := bottom_panel.find_child("Output", false, false)
	var scroll_bar := output_panel.find_child("*VScrollBar*", true, false)
	output_panel.visible = true
	# Scroll to bottom.
	scroll_bar.value = scroll_bar.max_value
