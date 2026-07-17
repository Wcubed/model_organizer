extends EditorExportPlugin
class_name RustExportPlugin

func _get_name() -> String:
	return "RustExportPlugin"

func _export_begin(features: PackedStringArray, is_debug: bool, path: String, flags: int) -> void:
	if features.find("linux") != -1:
		execute_build_command("./rust_build_linux.sh")
	if features.find("windows") != -1:
		execute_build_command("./rust_build_windows.sh")


## Executes the build command and keeps updating the editor while the command is running.
func execute_build_command(command: String) -> bool:
	print("Building rust addon...")
	
	var handles = OS.execute_with_pipe(command, [], false)
	var stdio: FileAccess = handles["stdio"]
	var stderr: FileAccess = handles["stderr"]
	var pid: int = handles["pid"]
	
	# Show the output panel.
	var bottom_panel := EditorInterface.get_base_control().find_child("*EditorBottomPanel*", true, false)
	var output_panel := bottom_panel.find_child("Output", false, false)
	var scroll_bar := output_panel.find_child("*VScrollBar*", true, false)
	output_panel.visible = true
	
	var last_update := Time.get_unix_time_from_system()
	
	var io_out := ""
	var err_out := ""
	
	while OS.is_process_running(pid):
		io_out += stdio.get_as_text()
		err_out += stderr.get_as_text()
		
		if io_out.ends_with("\n"):
			print(io_out.trim_suffix("\n"))
			io_out = ""
		if err_out.ends_with("\n"):
			print(err_out.trim_suffix("\n"))
			err_out = ""
		
		DisplayServer.process_events()
		RenderingServer.force_draw()
	
	io_out += stdio.get_as_text()
	err_out += stderr.get_as_text()
	
	print(io_out)
	print(err_out)
	
	var result := OS.get_process_exit_code(pid)
	return result == 0


func build_rust_linux() -> bool:
	var output = []
	var result := OS.execute("./rust_build_linux.sh", [], output, true, true)
	
	if result != 0:
		output_cargo_error(output[0])
	
	return result == 0


func build_rust_windows() -> bool:
	var output = []
	var result := OS.execute("./rust_build_windows.sh", [], output, true, true)
	
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
