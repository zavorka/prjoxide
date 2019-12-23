"""
Utilities for fuzzing interconect
"""

import threading
import tiles
import libprjoxide
import fuzzconfig
import fuzzloops
import lapie

def fuzz_interconnect(
        config,
        nodenames,
        regex=False,
        nodename_predicate=lambda x, nets: True,
        pip_predicate=lambda x, nets: True,
        bidir=False,
        nodename_filter_union=False,
        full_mux_style=False,
        ignore_tiles=set()
    ):
    """
    Fuzz interconnect given a list of nodenames to analyse. Pips associated these nodenames will be found using the Tcl
    API and bits identified as described above.

    :param config: FuzzConfig instance containing target device and tile(s) of interest
    :param nodenames: A list of nodes or node regexes in Lattice (un-normalised) format to analyse
    :param regex: enable regex names
    :param nodename_predicate: a predicate function which should return True if a netname is of interest, given
    the netname and the set of all nets
    :param pip_predicate: a predicate function which should return True if an arc, given the arc as a (source, sink)
    tuple and the set of all nodenames, is of interest
    :param bidir: if True, pips driven by as well as driving the given nodenames will be considered during analysis
    :param nodename_filter_union: if True, pips will be included if either net passes nodename_predicate, if False both
    nets much pass the predicate.
    :param full_mux_style: if True, is a full mux, and all 0s is considered a valid config bit possibility
    on certain families.
    """
    nodes = lapie.get_node_data(config.udb, nodenames, regex)
    base_bitf = config.build_design(config.sv, {"arcs_attr": ""}, "base_")

    all_wirenames = set([n.name for n in nodes])

    def per_node(node):
        # Get a unique prefix from the thread ID
        prefix = "thread{}_".format(threading.get_ident())
        assoc_pips = set()
        for p in node.uphill_pips:
        	assoc_pips.add((p.from_wire, p.to_wire))
        if bidir:
        	for p in node.downhill_pips:
        		assoc_pips.add((p.from_wire, p.to_wire))
        assoc_pips = list(sorted(assoc_pips))
        # First filter using netname predicate
        if nodename_filter_union:
            assoc_pips = filter(lambda x: nodename_predicate(x[0], all_wirenames) and nodename_predicate(x[1], all_wirenames),
                                assoc_pips)
        else:
            assoc_pips = filter(lambda x: nodename_predicate(x[0], all_wirenames) or nodename_predicate(x[1], all_wirenames),
                                assoc_pips)
        # Then filter using the pip predicate
        fuzz_pips = list(filter(lambda x: pip_predicate(x, all_wirenames), assoc_pips))
        if len(fuzz_pips) == 0:
        	return
        sinks = {}
        for from_wire, to_wire in fuzz_pips:
        	if to_wire not in sinks:
        		sinks[to_wire] = []
        	sinks[to_wire].append(from_wire)
        for to_wire in sorted(sinks.keys()):
        	fz = libprjoxide.Fuzzer.pip_fuzzer(fuzzconfig.db, base_bitf, set(config.tiles), to_wire, config.tiles[0], ignore_tiles, full_mux_style, False)
        	for from_wire in sinks[to_wire]:
        		arcs_attr = r', \dm:arcs ="{}.{}"'.format(to_wire, from_wire)
        		arc_bit = config.build_design(config.sv, {"arcs_attr": arcs_attr}, prefix)
        		fz.add_pip_sample(fuzzconfig.db, from_wire, arc_bit)
        	fz.solve(fuzzconfig.db)
    fuzzloops.parallel_foreach(nodes, per_node)
