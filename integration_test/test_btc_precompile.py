import requests

def test_btc_precompile_single_vin_vout():
    response = requests.post(
        "http://localhost:18545",
        json={
            "jsonrpc": "2.0",
            "method": "eth_call",
            "params": {
                "from": "0x1234567890123456789012345678901234567890",
                "to": "0x00000000000000000000000000000000000000fd",
                # https://mempool.space/testnet4/tx/cedfb4b62224a4782a4453dff73f3d48bb0d7da4d0f2238b0e949f9342de038a
                "data": "96327323000000000000000000000000000000000000000000000000000000000000004063656466623462363232323461343738326134343533646666373366336434386262306437646134643066323233386230653934396639333432646530333861",
            },
            "id": 65,
        },
    )
    # Test for the decoded values of this result is in server/src/evm/btc_precompile.rs
    assert response.status_code == 200
    assert (
        response.json()["result"]["resultBytes"]
        == "0x0000000000000000000000000000000000000000000000000000000000011f2d00000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000018000000000000000000000000000000000000000000000000000000000000001c0000000000000000000000000000000000000000000000000000000000000028000000000000000000000000000000000000000000000000000000000000002c000000000000000000000000000000000000000000000000000000000000003800000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000403366633564343962616439326131626331306339623039383166313363343736303061663837316464633962616331356432326432633035623661653830663100000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000044353132303135346137636266383361356164393239653232343732356239313563613563643763613737313961396261326166393039343561373465333433313933346400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000100000000000000000000000000000000000000000000000000000000000001e70000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000443531323066636463356137626436366234643361386339316631613163663934616437643536316633613330346266313866616635363738623165653437653738336237000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000000014a"
    )
    assert (
        response.json()["result"]["gasUsed"]
        == "0x36414"
    )


def test_btc_precompile_multiple_vin_vout():
    response = requests.post(
        "http://localhost:18545",
        json={
            "jsonrpc": "2.0",
            "method": "eth_call",
            "params": {
                "from": "0x1234567890123456789012345678901234567890",
                "to": "0x00000000000000000000000000000000000000fd",
                # https://mempool.space/testnet4/tx/ce1d2d142eb12fa4fbbb2c361c286483e5c74ca67640496de23beb5ee56d0406
                "data": "0x96327323000000000000000000000000000000000000000000000000000000000000004063653164326431343265623132666134666262623263333631633238363438336535633734636136373634303439366465323362656235656535366430343036",
            },
            "id": 65,
        },
    )
    # Test for the decoded values of this result is in server/src/evm/btc_precompile.rs
    assert response.status_code == 200
    assert (
        response.json()["result"]["resultBytes"]
        == "0x0000000000000000000000000000000000000000000000000000000000011d7600000000000000000000000000000000000000000000000000000000000000e0000000000000000000000000000000000000000000000000000000000000028000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000000500000000000000000000000000000000000000000000000000000000000000058000000000000000000000000000000000000000000000000000000000000007e00000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000012000000000000000000000000000000000000000000000000000000000000000403162613331333332323662313866313865636339653333333539613063356131323433323033613734666334666466383839386639653730396138323630616100000000000000000000000000000000000000000000000000000000000000406561326135353337343733336433633336313432373235623366343538353762663464373862323931353365613464636466386162356238383062343934663800000000000000000000000000000000000000000000000000000000000000406233663138663062343139656335653435323731363737373633343930656637626532653662346264353531396462613932346564316333316537383764346400000000000000000000000000000000000000000000000000000000000000030000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000000000000000000200000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000003000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000e00000000000000000000000000000000000000000000000000000000000000160000000000000000000000000000000000000000000000000000000000000004435313230353436656231386135643435396262353964393637396665386638643539386662663735363862663035636464613361663662323631386238666438633366340000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000443531323035343665623138613564343539626235396439363739666538663864353938666266373536386266303563646461336166366232363138623866643863336634000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000044353132303534366562313861356434353962623539643936373966653866386435393866626637353638626630356364646133616636623236313862386664386333663400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000300000000000000000000000000000000000000000000000000000000000002220000000000000000000000000000000000000000000000000000000000000222000000000000000000000000000000000000000000000000000000000012cb220000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000008000000000000000000000000000000000000000000000000000000000000000c0000000000000000000000000000000000000000000000000000000000000014000000000000000000000000000000000000000000000000000000000000001c0000000000000000000000000000000000000000000000000000000000000001c36613564306231363031303065393964303431656633383436613032000000000000000000000000000000000000000000000000000000000000000000000044353132303534366562313861356434353962623539643936373966653866386435393866626637353638626630356364646133616636623236313862386664386333663400000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004435313230623430633036356266636335393632653137303266303964653161356432646663306137323336626261663563313637323532396234313462336565346366350000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000443531323035343665623138613564343539626235396439363739666538663864353938666266373536386266303563646461336166366232363138623866643863336634000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000002220000000000000000000000000000000000000000000000000000000000000222000000000000000000000000000000000000000000000000000000000012b211"
    )
    assert (
        response.json()["result"]["gasUsed"]
        == "0x67154"
    )


if __name__ == "__main__":
    test_btc_precompile_single_vin_vout()
    test_btc_precompile_multiple_vin_vout()
