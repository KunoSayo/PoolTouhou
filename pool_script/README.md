# PoolScript

### Begin Pointer Data
* B0: const value
* B1: game value
* B2: script data value
* B3: script stack value
* B4: push expr
* B5: pop expr

### Begin Game Data
* B0: pos_x
* B1: pos_y
* B2: pos_z
* B3: player_x
* B4: player_y
* B5: player_z

### Begin File Data
* 4B : Version  
* 1B : f32 Data Count  
* 2B : Function Name Bytes
* Function Data
* B0: end declare
* B1: loop_begin (i32 as times)
* B2: ret
* B3: push_to_stack_top (pointer)
* B4: allocate f32

* B10: move_up (f32)
* B11: summon_e (name, xyz, hp, 1B sp_count, n...sp_name)
* B12: summon_b (name, xyz, angle, collide_name, args..., bullet_ai, args...)

* B20: store_f32 (pointer)
* B21: add
* B22: sub
* B23: mul
* B24: eq
* B25: neq
* B26: lt
* B27: gt