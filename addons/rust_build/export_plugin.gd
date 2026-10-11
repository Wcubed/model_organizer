extends EditorExportPlugin
class_name RustExportPlugin

## Set this when instantiating.
var scene_tree: SceneTree = null
## When a build is running, this will contain the process handles.
var handles := {}
## Contains the exit code from the last build.
var exit_code := 0

func _init(tree: SceneTree) -> void:
	scene_tree = tree

func _get_name() -> String:
	return "RustExportPlugin"

func _export_begin(features: PackedStringArray, is_debug: bool, path: String, flags: int) -> void:
	if features.find("linux") != -1:
		start_build_command("./rust_build_linux.sh")
	if features.find("windows") != -1:
		start_build_command("./rust_build_windows.sh")
	
	# Wait for the build to finish.
	while update_running_build():
		OS.delay_msec(500)


func start_build_command(command: String):
	if not handles.is_empty():
		# A build is already running.
		return
	
	print()
	print()
	print_rich("[color=web_gray]--- Rust build started ---[/color]")
	
	handles = OS.execute_with_pipe(command, [], false)
	
	# Show the output panel.
	var bottom_panel := EditorInterface.get_base_control().find_child("*EditorBottomPanel*", true, false)
	var output_panel := bottom_panel.find_child("Output", false, false)
	output_panel.visible = true


## Call this every _process while the build is running.
## Returns true as long as the build is running.
func update_running_build() -> bool:
	if handles.is_empty():
		return false
	
	var stdio: FileAccess = handles["stdio"]
	var stderr: FileAccess = handles["stderr"]
	var pid: int = handles["pid"]
	_maybe_print_line(stdio)
	_maybe_print_line(stderr)
	
	if not OS.is_process_running(pid):
		handles = {}
		exit_code = OS.get_process_exit_code(pid)
		
		_maybe_print_line(stdio)
		_maybe_print_line(stderr)
		
		if exit_code == 0:
			print_rich("[color=green]--- Build complete ---[/color]")
		else:
			printerr("Build failed with code: ", exit_code)
		
		return false
	else:
		return true


func _maybe_print_line(stream: FileAccess):
	var line := stream.get_line()
	if not line.is_empty():
		print(line)


func is_build_running() -> bool:
	return not self.handles.is_empty()
