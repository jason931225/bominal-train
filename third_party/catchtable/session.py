import json
import requests

class Session:
    def __init__(self, cookie):
        self.cookie = cookie

    def _get_headers(self, headers):
        return {
            'Host':'app.catchtable.co.kr',
            'Connection':'keep-alive',
            'Accept':'application/json, text/plain, */*',
            'User-Agent':'Mozilla/5.0 (Linux; Android 12; SM-A315N Build/SP1A.210812.016; wv) AppleWebKit/537.36 (KHTML, like Gecko) Version/4.0 Chrome/108.0.5359.128 Mobile Safari/537.36',
            'Content-Type':'application/json',
            'Origin':'https://app.catchtable.co.kr',
            'X-Requested-With':'co.kr.catchtable.android.catchtable_app',
            'Sec-Fetch-Site':'same-origin',
            'Sec-Fetch-Mode':'cors',
            'Sec-Fetch-Dest':'empty',
            'Accept-Encoding':'gzip, deflate',
            'Accept-Language':'ko-KR,ko;q=0.9,en-US;q=0.8,en;q=0.7',
            'Cookie': self.cookie,
            **headers,
        }

    def post(self, url, headers, payload):
        payload = json.dumps(payload)
        response = requests.request(
            "POST",
            f"https://app.catchtable.co.kr/api/v3/{url}",
            headers=self._get_headers(headers),
            data=payload
        )
        data = response.json().get("data")
        if data:
            return data
        return response.json()
