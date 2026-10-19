"""Transport helper unit tests; run with the XMPP requirements installed."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from slixmpp import JID

spec=importlib.util.spec_from_file_location('bridge',Path(__file__).parent.parent/'src/xmpp_bridge.py')
bridge=importlib.util.module_from_spec(spec)
spec.loader.exec_module(bridge)


class BridgeTests(unittest.TestCase):
    def test_grouped_fingerprints_and_invalid_entries(self):
        fingerprint='0123456789abcdef'*4
        grouped=' '.join(fingerprint[i:i+8] for i in range(0,64,8))
        pins=bridge.parse_pins(['alice@example.org '+grouped],JID)
        self.assertIn(('alice@example.org',bytes.fromhex(fingerprint)),pins)
        for entry in ['alice@example.org/device '+fingerprint,'alice@example.org bad','example.org '+fingerprint]:
            with self.assertRaises((ValueError,TypeError)):
                bridge.parse_pins([entry],JID)

    def test_sqlite_persists_and_refuses_final_symlink(self):
        with tempfile.TemporaryDirectory() as root:
            path=Path(root)/'state'
            db=bridge.private_database(path)
            with db:db.execute('insert into state values (?,?)',('fixture',json.dumps('persistent')))
            db.close()
            db=bridge.private_database(path)
            self.assertEqual(json.loads(db.execute("select value from state where key='fixture'").fetchone()[0]),'persistent')
            db.close()
            self.assertEqual((path/'omemo.sqlite3').stat().st_mode & 0o777,0o600)
            link=Path(root)/'link';link.symlink_to(path,target_is_directory=True)
            with self.assertRaises(ValueError):bridge.private_database(link)


if __name__=='__main__':unittest.main()
