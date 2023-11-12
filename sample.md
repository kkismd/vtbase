```asm
     1                          CHROUT=$ffd2           ; chrout :=$ffd2             ; CHROUT EQU $ffd2
     2                                *=$080e          ;        *=$080e             ;        start $080e
     3                                                   
     4  080e a200                     ldx #$00         ;        X=#0                ;        X=#0
     5  0810 bd1c08             loop  lda hello, x     ; loop   A=hello+X           ; loop   A=hello+X
     6  0813 20d2ff                   jsr CHROUT       ;        !=chrout            ;        GOSUB CHROUT
     7  0816 e8                       inx              ;        X=+                 ;        X++
     8  0817 e00d                     cpx #$0d         ;        ;=X~#$0d #=loop     ;        T=X-#$0d
     9  0819 d0f5                     bne loop         ;                            ;        IF <> GOTO loop
    10  081b 60                       rts              ;        #=!                 ;        RETURN
    11                                                   
    12                          hello                  ; hello                      ; hello
    13  081c 48454c4c4f2c2057...    !tx "HELLO, WORLD.";        ?="HELLO, WORLD."   ;        !("HELLO, WORLD.")
    14                                ;  0123456789012
```

ADC  A=A+n
AND  A=A&n
ASL  <=A
BCC  ;=< #=$1234      ; IF < GOTO $1234
BCS  ;=> #=$1234      ; IF >= GOTO $1234
BEQ  ;== #=$1234      ; IF = GOTO $1234
BMI  ;=- #=$1234      ; IF - GOTO $1234
BNE  ;=/ #=$1234      ; IF <> GOTO $1234
BPL  ;=+ #=$1234      ; IF + GOTO $1234
BVC  ;=_ #=$1234      ; IF V=0 GOTO $1234
BVS  ;=^ #=$1234      ; IF V=1 GOTO $1234
BIT  T=A&$1234
BRK  !
CLC  C=0
CLD  D=0
CLI  I=0
CLV  V=0
CMP  T=A-n
CPX  T=X-n
CPY  T=Y-n
DEC  A=-
DEX  X=-
DEY  Y=-
EOR  A=A^n
INC  A=+
INX  X=+
INY  Y=+
JMP  #=$1234
JSR  !=$1234
LDA  A=1
LDX  X=1
LDY  Y=1
LSR  >=A
NOP  -
ORA  A=A|1
PHA  [=A
PHP  [=P
PLA  A=]
PLP  P=]
ROL  A<<
ROR  A>>
RTI  ~
RTS  ^
SBC  A=A-n
SEC  C=1
SED  D=1
SEI  I=1
STA  ($1234)=A
STX  ($1234)=X
STY  ($1234)=Y
TAX  X=A
TAY  Y=A
TSX  X=S
TXA  A=X
TXS  S=X
TYA  A=Y

# マクロ

```vtl
  A<$12 ...      ; IF A<$12 THEN ...
```

展開イメージ

```vtl
  T=A-n
  ;=< #=.next_label
  ...
.next_label
```


# アドレッシングモード

| mode                | asm format | vtl format |
| ------------------- | ---------- | ---------- |
| Implied             |            |            |
| Immediate           | #aa        | aa         |
| Absolute            | aaaa       | (aaaa)     |
| Relative            | aaaa       | (aaaa)     |
| Zero Page           | aa         | (aa)       |
| Absolute Indexed,X  | aaaa,X     | (aaaa+X)   |
| Absolute Indexed,Y  | aaaa,Y     | (aaaa+Y)   |
| Zero Page Indexed,X | aa,X       | (aa,X)     |
| Zero Page Indexed,Y | aa,Y       | (aa,Y)     |
| Indirect Absolute   | (aaaa)     | [awaa]     |
| Indexed Indirect    | (aa,X)     | [aa,X]     |
| Indirect Indexed    | (aa),Y     | [aa],Y     |
| Accumulator         | A          | A          |

Note:
aa = 2 hex digits as $FF
aaaa = 4 hex digits as $FFFF

Can also be assembler labels

## オペランド記法

```vtl
A=#label  ; ラベル
A=$FF     ; 即値
A=($FF)   ; ゼロページ
A=($1234) ; アブソリュート
```

```asm
A=#label
A=#$FF
A=$FF
A=$1234
```

# 課題メモ

* ラベル定義をどうするか
  * ラベルは小文字,アンダースコア,数字
  * EQU疑似命令に相当
  * "ラベル :=アドレス"
  * "chrout :=$ffd2"
* データ定義文をどうするか
  * 命令の記号は ?, $, %
  * ?="Hello, World"  ASCIIデータ
  * %={"string" $0A $0D $3F1D $FFFF} 混合データ文 文字列、バイト、ワード
* IF 文のブロックをどうするか
  * 「前の行からの続き」という形でブロックの代わりとする
  * 継続行の命令は "==="
  * ;=Z A=#$10 !=subr1 A=A+#$10  !=subr1
    === A=A-#$10 !=subr1
    === A=A+#$20 !=subr1

記号使用状況

| 記号 | VTBase       | VTL,GAME80                 |
| ---- | ------------ | -------------------------- |
| !    | JSR          |                            |
| @    |              | LOOP (GAME80)              |
| #    | JMP or BRA   |                            |
| $    | バイトデータ |                            |
| &    |              | High memory (VTL)          |
| *    | ORG          | Low memory (VTL)           |
| (    |              |                            |
| )    |              | コメント (VTL)             |
| [    | PUSH         |                            |
| ]    | POP          | RETURN (GAME80)            |
| <    |              | File出力 or ポインタ (VTL) |
| >    |              | File入力 (VTL)             |
| /    |              | 改行 (GAME80)              |
| ?    | 文字列データ |                            |
| :    | EQL          | 配列 (VTL)                 |
| ;    | IF           |                            |
| %    | ワードデータ | 剰余 (VTL, GAME80)         |
| ^    | RTS          |                            |
| `    |              |                            |
| _    |              |                            |
| +    |              |                            |
| =    |              |                            |
| \    |              |                            |
| ,    |              |                            |
| .    |              | タブ出力                   |
| '    | コメント     | 乱数                       |
| "    |              | PRINT GAME80               |
| {    | REPEAT       |                            |
| }    | UNTIL        |                            |
| ~    | RTI          |                            |

# 演算子

| 記号 | 意味                                                                           |
| ---- | ------------------------------------------------------------------------------ |
| <    | 小なり                                                                         |
| >    | 大なりイコール                                                                 |
| =    | イコール                                                                       |
| #    | ノットイコール  -> 即値リテラルの # とつながると読みづらいので別の記号と変える |
| ~    | ノットイコール(新)                                                             |

