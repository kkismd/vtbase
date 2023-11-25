# vtbase

Very Tiny BASE for 6502

BASE like assembler using VTL like notation.

code image.

```asm
     1                          CHROUT=$ffd2           ; CHROUT :=$ffd2           
     2                                *=$080e          ;        *=$080e           
     3                                                   
     4  080e a200                     ldx #$00         ;        X=#0              
     5  0810 bd1c08             loop  lda hello,x      ; loop   A=hello+X         
     6  0813 20d2ff                   jsr CHROUT       ;        !=CHROUT          
     7  0816 e8                       inx              ;        X=+               
     8  0817 e00d                     cpx #$0d         ;        ;=X>13,loop   
     9  0819 d0f5                     bne loop         ;                          
    10  081b 60                       rts              ;        #=!               
    11                                                   
    12                          hello                  ; hello                    
    13  081c 48454c4c4f2c2057...    !tx "HELLO, WORLD.";        ?="HELLO, WORLD." 
```

## 6502 Mnemonic <-> VTBase Statement

| Mnemonic | VTL like statement | Note                                      |
| -------- | ------------------ | ----------------------------------------- |
| ADC      | A=A+n              | CLC ADC のマクロ命令                      |
| ADC      | A=AC+n             | キャリークリアをしない場合                |
| AND      | A=A&n              |                                           |
| ASL      | <=A                |                                           |
| ASL      | A=A[1              | 文法を単純にするため演算子は1文字にしたい |
| BCC      | ;=<,$1234          | IF < GOTO $1234                           |
| BCS      | ;=>,$1234          | IF >= GOTO $1234                          |
| BEQ      | ;==,$1234          | IF = GOTO $1234                           |
| BMI      | ;=-,$1234          | IF - GOTO $1234                           |
| BNE      | ;=\,$1234          | IF <> GOTO $1234                          |
| BPL      | ;=+,$1234          | IF + GOTO $1234                           |
| BVC      | ;=_,$1234          | IF V=0 GOTO $1234                         |
| BVS      | ;=^,$1234          | IF V=1 GOTO $1234                         |
| BIT      | T=A&$1234          |                                           |
| BRK      | !                  |                                           |
| CLC      | C=0                |                                           |
| CLD      | D=0                |                                           |
| CLI      | I=0                |                                           |
| CLV      | V=0                |                                           |
| CMP      | T=A-n              |                                           |
| CPX      | T=X-n              |                                           |
| CPY      | T=Y-n              |                                           |
| DEC      | A=-                |                                           |
| DEX      | X=-                |                                           |
| DEY      | Y=-                |                                           |
| EOR      | A=A^n              |                                           |
| INC      | A=+                |                                           |
| INX      | X=+                |                                           |
| INY      | Y=+                |                                           |
| JMP      | #=$1234            |                                           |
| JSR      | !=$1234            |                                           |
| LDA      | A=1                | immediate                                 |
| LDA      | A=($1F0A)          | absolute                                  |
| LDA      | A=($0A)            | zero page                                 |
| LDA      | A=[$1F0A]          | indirect absolute                         |
| LDA      | A=[$0A+X]          | indexed indirect                          |
| LDA      | A=[$0A]+Y          | indirect indexed                          |
| LDX      | X=1                |                                           |
| LDY      | Y=1                |                                           |
| LSR      | >=A                |                                           |
| LSR      | A=A]1              |                                           |
| NOP      | .                  |                                           |
| ORA      | A=A                |                                           |
| PHA      | [=A                |                                           |
| PHP      | [=P                |                                           |
| PLA      | A=]                |                                           |
| PLP      | P=]                |                                           |
| ROL      | (=A                |                                           |
| ROR      | )=A                |                                           |
| ROL      | A=A{1              |                                           |
| ROR      | A=A}1              |                                           |
| RTI      | ~                  |                                           |
| RTS      | #=!                |                                           |
| RTS      | !                  |                                           |
| RTS      | ^                  |                                           |
| RTS      | ]                  |                                           |
| SBC      | A=A-n              | SEC SBC のマクロ命令                      |
| SBC      | A=AC-n             | キャリーセットをしない場合                |
| SEC      | C=1                |                                           |
| SED      | D=1                |                                           |
| SEI      | I=1                |                                           |
| STA      | ($1234)=A          |                                           |
| STX      | ($1234)=X          |                                           |
| STY      | ($1234)=Y          |                                           |
| TAX      | X=A                |                                           |
| TAY      | Y=A                |                                           |
| TSX      | X=S                |                                           |
| TXA      | A=X                |                                           |
| TXS      | S=X                |                                           |
| TYA      | A=Y                |                                           |

## Macro statement

IFマクロ

```vtl
  ;=A<$12 ...
```

展開イメージ

```vtl
  T=A-n
  ;=<,.next_label
  ...
.next_label
```

DO-WHILEマクロ

```vtl
  @
    ...
  @=X>0
```

展開イメージ

```vtl
.loop_label
  ...
  T=X-0
  ;=<,.loop_label
```


## Addressing Mode

| mode                | asm format | vtbase format |
| ------------------- | ---------- | ------------- |
| Implied             |            |               |
| Immediate           | #aa        | aa            |
| Absolute            | aaaa       | (aaaa)        |
| Relative            | aaaa       | (aaaa)        |
| Zero Page           | aa         | (aa)          |
| Absolute Indexed,X  | aaaa,X     | (aaaa+X)      |
| Absolute Indexed,Y  | aaaa,Y     | (aaaa+Y)      |
| Zero Page Indexed,X | aa,X       | (aa+X)        |
| Zero Page Indexed,Y | aa,Y       | (aa+Y)        |
| Indirect Absolute   | (aaaa)     | [aaaa]        |
| Indexed Indirect    | (aa,X)     | [aa+X]        |
| Indirect Indexed    | (aa),Y     | [aa]+Y        |
| Accumulator         | A          | A             |

Note:
* aa = 2 hex digits as $FF
* aaaa = 4 hex digits as $FFFF

## Pseudo Command

| asm                   | vtl like          |
| --------------------- | ----------------- |
| LABEL EQU $aaaa       | label :=$aaa      |
| *=$aaaa               | *=$aaaa           |
| .text "hello world",0 | ?="hello world",0 |


## 課題メモ

* IF 文のブロックをどうするか
  * 「前の行からの続き」という形でブロックの代わりとする
  * 継続行の命令は "==="
  * ;=Z A=#$10 !=subr1 A=A+#$10  !=subr1
    === A=A-#$10 !=subr1
    === A=A+#$20 !=subr1

記号使用状況

| 記号 | command      | expression     | VTL,GAME80                 |
| ---- | ------------ | -------------- | -------------------------- |
| !    | JSR          |                |                            |
| @    | LOOP         |                | LOOP (GAME80)              |
| #    | JMP or BRA   |                |                            |
| $    | バイトデータ |                |                            |
| &    | データ領域   | AND            | High memory (VTL)          |
| *    | ORG          |                | 乗算 / Low memory (VTL)    |
| (    | ローテート左 | 括弧           |                            |
| )    | ローテート右 |                | コメント (VTL)             |
| [    | PUSH         | 論理シフト左   |                            |
| ]    | POP          | 論理シフト右   | RETURN (GAME80)            |
| <    | 論理シフト左 | 小なり         | File出力 or ポインタ (VTL) |
| >    | 論理シフト右 | 大なりイコール | File入力 (VTL)             |
| /    |              | 除算           | 改行 (GAME80)              |
| ?    | 文字列データ |                |                            |
| :    | EQL          | ELSE           | 配列 (VTL)                 |
| ;    | IF           |                |                            |
| %    |              |                | 剰余(VTL, GAME80)          |
| ^    | RTS          | XOR            |                            |
| ｜   |              | OR             |                            |
| ｀   |              |                |                            |
| _    | NOP          |                |                            |
| -    | NOP          | 減算           |                            |
| +    |              | 加算           |                            |
| =    |              | 等号           |                            |
| \    |              | 不等号         |                            |
| ,    |              | COMMA          |                            |
| .    |              |                | タブ出力                   |
| '    | コメント     |                | 乱数                       |
| "    |              | 文字列         | PRINT GAME80               |
| {    |              | ローテート左   |                            |
| }    |              | ローテート右   |                            |
| ~    | RTI          | 不等号         |                            |


