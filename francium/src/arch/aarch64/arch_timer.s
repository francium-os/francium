.global get_cntfrq
.global set_cntp_tval_el0
.global set_cntp_ctl_el0

get_cntfrq:
    mrs x0, CNTFRQ_EL0
    ret

set_cntp_tval_el0:
    msr CNTP_TVAL_EL0, x0
    ret

set_cntp_ctl_el0:
    msr CNTP_CTL_EL0, x0
    ret