/*
Copyright 2022 @Yowkees
Copyright 2022 MURAOKA Taro (aka KoRoN, @kaoriya)

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 2 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#include QMK_KEYBOARD_H

#include "quantum.h"

// clang-format off
const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
    // Base Layer
  [0] = LAYOUT_universal(
    KC_TAB   , KC_Q     , KC_W     , KC_E     , KC_R     , KC_T     ,                                        KC_Y     , KC_U     , KC_I     , KC_O     , KC_P     , KC_BSPC   ,
    KC_LCTL  , KC_A     , KC_S     , KC_D     , KC_F     , KC_G     ,                                        KC_H     , KC_J     , KC_K   , LT(5, KC_L), KC_SCLN  , KC_ENT  ,
    KC_LSFT  , KC_Z     , KC_X     , KC_C     , KC_V     , KC_B     ,                                        KC_N     , KC_M     , KC_COMM  , KC_DOT   , KC_SLSH  , KC_RSFT  ,
                  MO(4),KC_LALT,KC_LGUI     ,MO(3),KC_SPC,                                        MO(1),MO(2), KC_RALT,     _______  , MO(5)
  ),

    // Symbol Layer
  [1] = LAYOUT_universal(
    _______ ,  KC_GRV   ,S(KC_BSLS), _______  , KC_QUOT  ,S(KC_QUOT),                                        _______  ,S(KC_LBRC),S(KC_RBRC), KC_LBRC  , KC_RBRC  , _______  ,
    _______ , S(KC_1)   ,  S(KC_2) , S(KC_3)  , S(KC_4)  , S(KC_5)  ,                                         S(KC_6) , S(KC_7)  , S(KC_8)  , S(KC_9)  , S(KC_0)  , _______  ,
    _______ ,  _______  , _______  , _______  , _______  , _______  ,                                         _______ , _______  , _______  , _______  , KC_BSLS  , _______  ,
                _______,_______,_______     ,_______,_______,                                     _______,_______,_______,   _______  , _______
  ),

    // Number Layer
  [2] = LAYOUT_universal(
    _______ , S(KC_GRV) , _______  , _______  , _______  , _______  ,                                         _______ , KC_MINUS ,S(KC_MINUS), KC_EQL  , S(KC_EQL), _______  ,
    _______ , KC_1      , KC_2     , KC_3     , KC_4     , KC_5     ,                                         KC_6    , KC_7     , KC_8      , KC_9    , KC_0     , _______  ,
    _______ , _______   , _______  , _______  , _______  , _______  ,                                        _______  ,_______   , _______   ,_______  ,_______   ,_______   ,
                _______,_______,_______     ,_______,_______,                                     _______,_______,_______,   _______  , _______
  ),

    // Navigation Layer
  [3] = LAYOUT_universal(
    KC_ESC  , _______   , _______  , _______  , _______  , _______  ,                                        _______  , KC_PGUP  , KC_PGDN   , KC_HOME , KC_END   , KC_DEL   ,
    _______ , _______   , _______  , _______  , _______  , _______  ,                                        _______  , KC_LEFT  , KC_DOWN   , KC_UP   , KC_RIGHT , S(KC_INSERT),
    _______ , _______   , _______  , _______  , _______  , _______  ,                                        _______  ,_______   , _______   ,_______  ,_______   ,_______   ,
                _______,_______,_______     ,_______,_______,                                     _______,_______,_______,   _______  , _______
  ),

    // Function Layer
  [4] = LAYOUT_universal(
    _______ , KC_F11    , KC_F12   , _______  , _______  , _______  ,                                        _______  , _______  , KC_PSCR   , KC_SCRL , KC_PAUSE , _______  ,
    _______ , KC_F1     , KC_F2    , KC_F3    , KC_F4    , KC_F5    ,                                        KC_F6    , KC_F7    , KC_F8     , KC_F9   , KC_F10   , _______  ,
    _______ , KC_CAPS   , KC_NUM   , _______  , _______  , _______  ,                                        _______  , _______  , _______   , _______ , _______  , _______  ,
                _______,_______,_______     ,_______,_______,                                     _______,_______,_______,   _______  , _______
  ),

    // Trackball Layer
  [5] = LAYOUT_universal(

    _______  , AML_TO   , AML_I50  , AML_D50  , _______  , _______  ,                                        _______  , _______  , _______  , _______ , _______ , _______  ,
    _______  , _______  , _______  , _______  , _______  , SCRL_DVI ,                                        _______  , KC_BTN1  , KC_BTN2  , _______ , _______  , _______  ,
    _______ , _______  , _______  , _______  , _______  , SCRL_DVD ,                                        CPI_D1K  , CPI_D100 , CPI_I100 , CPI_I1K  , _______  , KBC_SAVE ,
                  QK_BOOT  , KBC_RST  , _______  ,        _______  , _______  ,                   _______  , _______  , _______       , KBC_RST  , QK_BOOT
  ),
};
// clang-format on

layer_state_t layer_state_set_user(layer_state_t state) {
  // Auto enable scroll mode when the highest layer is 3
  keyball_set_scroll_mode(get_highest_layer(state) == 3);
  return state;
}

#ifdef OLED_ENABLE

#include "lib/oledkit/oledkit.h"

void oledkit_render_info_user(void) {
  keyball_oled_render_keyinfo();
  keyball_oled_render_ballinfo();
  keyball_oled_render_layerinfo();
}
#endif
