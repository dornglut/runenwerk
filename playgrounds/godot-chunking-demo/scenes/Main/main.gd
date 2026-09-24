extends Node3D

@onready var streamer = $ChunkStreamingNode
@onready var player = $Player

func _ready():
	streamer.chunk_entered.connect(_on_chunk_entered)
	streamer.chunk_exited.connect(_on_chunk_exited)
	streamer.active_chunk_count_changed.connect(_on_active_chunk_count_changed)

func _process(_delta):
	streamer.update_focus_from_vector3(player.global_position)

func _on_chunk_entered(x: int, y: int, z: int) -> void:
	print("chunk entered: ", x, ", ", y, ", ", z)

func _on_chunk_exited(x: int, y: int, z: int) -> void:
	print("chunk exited: ", x, ", ", y, ", ", z)

func _on_active_chunk_count_changed(count: int) -> void:
	print("active chunk count: ", count)
