#!/bin/bash

radiantdir="${RADIANTDIR:-$HOME/lscc/radiant/2.0}"
export FOUNDRY="${radiantdir}/ispfpga"
bindir="${radiantdir}/bin/lin64"
LSC_DIAMOND=true
export LSC_DIAMOND
export NEOCAD_MAXLINEWIDTH=32767
export TCL_LIBRARY="${radiantdir}/tcltk/lib/tcl8.5"
export fpgabindir=${FOUNDRY}/bin/lin64
export LD_LIBRARY_PATH="${bindir}:${fpgabindir}"
export LM_LICENSE_FILE="${radiantdir}/license/license.dat"

set -ex

V_SUB=${2%.v}
PART=$1
set -- "$1" $V_SUB

case "${PART}" in
	LIFCL-40)
		PACKAGE="${DEV_PACKAGE:-CABGA400}"
		DEVICE="LIFCL-40"
		LSE_ARCH="lifcl"
		SPEED_GRADE="${SPEED_GRADE:-7_High-Performance_1.0V}"
		;;
esac

(

rm -rf "$2.tmp"
mkdir -p "$2.tmp"
cp "$2.v" "$2.tmp/input.v"

cd "$2.tmp"
"$fpgabindir"/synthesis -a "$LSE_ARCH" -p "$DEVICE" -t "$PACKAGE" \
			-use_io_insertion 1 -use_io_reg auto -use_carry_chain 1 \
			-ver input.v \
			-output_hdl synth.vm

"$fpgabindir"/postsyn -a "$LSE_ARCH" -p "$DEVICE" -t "$PACKAGE" -sp "$SPEED_GRADE" \
			-top -w -o synth.udb synth.vm

"$fpgabindir"/map -o map.udb synth.udb
"$fpgabindir"/par map.udb par.udb
if [ -n "$GEN_RBF" ]; then
"$fpgabindir"/bitgen -b -d -w par.udb
else
"$fpgabindir"/bitgen -d -w par.udb
fi
export LD_LIBRARY_PATH=""
)

if [ -n "$GEN_RBF" ]; then
cp "$2.tmp"/par.rbt "$2.rbt"
else
cp "$2.tmp"/par.bit "$2.bit"
fi