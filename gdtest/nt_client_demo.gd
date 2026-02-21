extends Node2D

# Create NodeTunnelPeer
var nt_peer := NodeTunnelPeer.new()

func _ready():
	# Initialize the peer
	var err = nt_peer.initialize("127.0.0.1:8080", "some_app_id")
	
	# Log any client-side errors
	if err != Error.OK:
		print("Encountered error: " + str(err))
	
	# Tell Godot to use the NodeTunnelPeer that we created
	multiplayer.multiplayer_peer = nt_peer

func _on_host_pressed() -> void:
	# Host a room when we press the host button
	nt_peer.host_room(false)

func _on_join_pressed() -> void:
	# Join a room when we press the join button
	nt_peer.join_room(%RoomId.text)
