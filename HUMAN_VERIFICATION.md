# Human verification (physical input — the agent cannot do these)

Run from the project folder: `cargo run --release`. Your save lives in the project root and is backed up to
`save_backups/` on every launch.

## H1 Keyboard, single player (R14)
1. Walk with WASD and arrows; hold Shift to run. Diagonals should not be faster than straight lines.
2. Face your home door and press **E** → fade into your room. Press **I** → pockets open; **Esc** closes.
3. Walk down through the doorway → back outside at your door.
Expected: all of the above works without stuck keys.

## H2 One controller joins as Player 2 (R10)
1. Connect a gamepad. Press **Start** (or **A**) on it.
2. Expected: the screen splits; the right half is Player 2 (red "P2" tag) standing at Player 2's Home.
3. Move with the left stick and d-pad; hold RT to run; press **A** at your door to enter; **Y** opens pockets;
   **B** closes; **X** uses the equipped tool; **RB** cycles tools; **Start** opens the journal.
4. In a menu, hold the stick down: the selection should repeat after a short delay.
5. Bottom hint line in P2's half should show controller buttons, not keyboard keys.

## H3 Independence (R09)
1. Player 1 (keyboard) opens pockets with **I** while Player 2 walks around: P2 must keep moving.
2. Player 2 talks to a resident (**A**); pressing **Esc**/**E** on the keyboard must not advance or close P2's dialogue.

## H4 Disconnect / reconnect (R11)
1. With Player 2 joined, unplug P2's controller. Expected: P2's half shows "Controller disconnected".
2. Plug it back in (or use a different controller) and press **Start**. Expected: control returns; P2's money,
   pockets and position are unchanged.
3. Press **F2** on the keyboard to make Player 2 leave; press **F2** again to rejoin: P2's progress is still there.

## H5 Two controllers
1. With two pads, the first pad pressing Start becomes Player 2, the second becomes an extra pad for Player 1.

Record results (pass/fail + notes) at the bottom of this file.

## Items 3/4/7 (2026-11-14)
- Notice board as P1: start a funded project, walk the green/red preview, place; donate from both players; check it appears the next morning.
- Town Hall door: desk opens; enact Dawn/Lantern/Tidy as P1 (P2 cannot); check store hours sign and residents' hours.
- Gullrock board: play Crab Corral, Beacon Dash, Ring Swim solo and with both players on the islet; check thresholds feel fair with real controllers (provisional values).
- Town Hall interior: walk in, talk to Hazel with keyboard and each gamepad; check menus in split-screen.
- Activity difficulty (all three) with real players: thresholds are provisional, assessment pending.
