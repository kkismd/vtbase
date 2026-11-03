# vtbase

## 開発・移行計画

現在の実装は独立した6502アセンブラです。64tass向け透過的プリプロセッサへの移行は [Issue #45](https://github.com/kkismd/vtbase/issues/45) で計画・検討しています。移行前の構文や実装をこのREADMEに記載したまま残しており、提案中の新版仕様を確定事項とは扱いません。

- [AGENTS.md](AGENTS.md) — Codexを含むエージェント共通の作業契約
- [docs/project-instructions.md](docs/project-instructions.md) — ChatGPT Project Instructions用の指示（ChatGPT Projectへの設定は別途必要）

Very Tiny BASE for 6502

BASE like assembler using VTL like notation.

code image.

```asm
     1                          CHROUT=$ffd2           ; CHROUT :=$ffd2           
     2                                *=$080e          ;        *=$080e           
     3                                                   
     4  080e a200                     ldx #$00         ;        X=0              
     5  0810 bd1c08             loop  lda hello,x      ; loop   A=(hello+X)       
     6  0813 20d2ff                   jsr CHROUT       ;        !=CHROUT          
     7  0816 e8                       inx              ;        X=+               
     8  0817 e00d                     cpx #$0d         ;        ;=X<14 #=loop
     9  0819 d0f5                     bne loop         ;         
    10  081b 60                       rts              ;        #=!               
    11                                                   
    12                          hello                  ; hello                    
    13  081c 48454c4c4f2c2057...    !tx "HELLO, WORLD.";        ?="HELLO, WORLD." 
```

## 6502 Mnemonic <-> VTBase Statement

| Mnemonic | VTL like statement | Note                      |
| -------- | ------------------ | ------------------------- |
| ADC      | A=A+n              | CLC ADC (macro command)   |
| ADC      | A=AC+n             | ADC (add with carry)      |
| AND      | A=A&n              |                           |
| ASL      | A=<                |                           |
| BCC      | ;=<,$1234          | IF < GOTO $1234           |
| BCS      | ;=>,$1234          | IF >= GOTO $1234          |
| BEQ      | ;==,$1234          | IF = GOTO $1234           |
| BMI      | ;=-,$1234          | IF - GOTO $1234           |
| BNE      | ;=\,$1234          | IF <> GOTO $1234          |
| BPL      | ;=+,$1234          | IF + GOTO $1234           |
| BVC      | ;=_,$1234          | IF V=0 GOTO $1234         |
| BVS      | ;=^,$1234          | IF V=1 GOTO $1234         |
| BIT      | T=A&$1234          |                           |
| BRK      | !                  |                           |
| CLC      | C=0                |                           |
| CLD      | D=0                |                           |
| CLI      | I=0                |                           |
| CLV      | V=0                |                           |
| CMP      | T=A-n              |                           |
| CPX      | T=X-n              |                           |
| CPY      | T=Y-n              |                           |
| DEC      | A=-                |                           |
| DEX      | X=-                |                           |
| DEY      | Y=-                |                           |
| EOR      | A=A^n              |                           |
| INC      | A=+                |                           |
| INX      | X=+                |                           |
| INY      | Y=+                |                           |
| JMP      | #=$1234            |                           |
| JSR      | !=$1234            |                           |
| LDA      | A=1                | immediate                 |
| LDA      | A=($1F0A)          | absolute                  |
| LDA      | A=($0A)            | zero page                 |
| LDA      | A=[$1F0A]          | indirect absolute         |
| LDA      | A=[$0A+X]          | indexed indirect          |
| LDA      | A=[$0A]+Y          | indirect indexed          |
| LDX      | X=1                |                           |
| LDY      | Y=1                |                           |
| LSR      | A=>                |                           |
| NOP      | .                  |                           |
| ORA      | A=A                |                           |
| PHA      | [=A                |                           |
| PHP      | [=P                |                           |
| PLA      | A=]                |                           |
| PLP      | P=]                |                           |
| ROL      | A=(                |                           |
| ROR      | A=)                |                           |
| RTI      | ~                  |                           |
| RTS      | #=!                |                           |
| RTS      | !                  |                           |
| RTS      | ^                  |                           |
| RTS      | ]                  |                           |
| SBC      | A=A-n              | SEC SBC (macro command)   |
| SBC      | A=AC-n             | SBC (subtract with carry) |
| SEC      | C=1                |                           |
| SED      | D=1                |                           |
| SEI      | I=1                |                           |
| STA      | ($1234)=A          |                           |
| STX      | ($1234)=X          |                           |
| STY      | ($1234)=Y          |                           |
| TAX      | X=A                |                           |
| TAY      | Y=A                |                           |
| TSX      | X=S                |                           |
| TXA      | A=X                |                           |
| TXS      | S=X                |                           |
| TYA      | A=Y                |                           |

## Macro statement

IF-macro

```vtl
  ;=A<$12 ...
```

expands to

```vtl
  T=A-n
  ;=<,.next_label
  ...
.next_label
```

DO-WHILE-macro

```vtl
  @
    ...
  @=X>0
```

expands to

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

## symbols

| 記号 | command      | expression        | VTL,GAME80                   |
| ---- | ------------ | ----------------- | ---------------------------- |
| !    | JSR          |                   |                              |
| @    | LOOP         |                   | LOOP (GAME80)                |
| #    | JMP or BRA   |                   |                              |
| $    | DATA Fill    | hex value         | print ascii code             |
| &    | Include Bin  | AND               | High memory (VTL)            |
| *    | ORG          | current addr      | multiply / Low memory (VTL)  |
| (    | Rotate Left  | Open Parentheses  |                              |
| )    | Rotate Right | Close Parentheses | comment (VTL)                |
| [    | PUSH         |                   |                              |
| ]    | POP          |                   | RETURN (GAME80)              |
| <    | Shift Left   | Less Than         | File output or pointer (VTL) |
| >    | Shift Right  | GTEq              | File input (VTL)             |
| /    |              | Div               | newline (GAME80)             |
| ?    | DATA         |                   | print string                 |
| :    | EQL          | ELSE              | Array :expr) (VTL)           |
| ;    | IF           | COMMENT           |                              |
| %    | MACRO        | bin value         | mod (VTL, GAME80)            |
| ^    |              | XOR               |                              |
| ｜   |              | OR                |                              |
| ｀   |              |                   |                              |
| _    | NOP          |                   |                              |
| -    |              | Sub               |                              |
| +    | Include SRC  | Add               |                              |
| =    |              | Equal             |                              |
| \    |              | Not Equal         |                              |
| ,    |              | COMMA             |                              |
| .    |              |                   | TAB (GAME80)                 |
| '    |              | CHAR              | random number generator      |
| "    |              | String            | PRINT GAME80                 |
| {    |              | Rotate LEft       |                              |
| }    |              | Rotate Right      |                              |
| ~    | RTI          | Not Equal         |                              |
