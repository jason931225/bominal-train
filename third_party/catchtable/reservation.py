class Reservation:
    def __init__(self, session, shop_ref, shop_alias, day, time):
        self.session = session
        self.shop_ref = shop_ref
        self.day = day
        self.time = time
        self.shop_alias = shop_alias

    def get_resp2(self, encrypted):
        headers = {
            'Referer':f'https://app.catchtable.co.kr/ct/shop/{self.shop_alias}"',
            "shopRef": self.shop_ref
        }
        data = self.session.post(url="reservation/availTimeSlots", headers=headers, payload=encrypted)

        return data.get("resp2")

    def prepare(self, resp2):
        payload = {
            "shopRef": self.shop_ref,
            "visitYymmdd": self.day,
            "visitHhmi": self.time,
            "personCount": 2,
            "menuSetSeq": 13388,
            "onlineNoticeSeq": 7697,
            "selectedMenus": [
                {
                    "menuItemSeq": "11090",
                    "count": 2
                }
            ],
            "prevHoldingSeq": 90904524,
            "tableTypes": [
                "_ALL_"
            ],
            "resp2": resp2,
            "precautionSeq": 5817
        } 

        headers = {
            'Referer':f'https://app.catchtable.co.kr/ct/shop/{self.shop_alias}"',
        }
        data = self.session.post(url="reservation/form", headers=headers, payload=payload)
        
        return data["token"]

    def reserve(self, token, resp2):
        payload = {
            "rccParam": {
                "shopRef": self.shop_ref,
                "clientLang": "KO",
                "isModifyCase": False,
                "isDiffVisitorBooker": False,
                "bookerName": "서우성",
                "bookerPhoneNumber": "01055359108",
                "bookerPhoneRegionCode": "KR",
                "visitorName": "",
                "visitorPhoneNumber": None,
                "visitorPhoneRegionCode": "KR",
                "visitYymmdd": self.day,
                "visitHhmi": self.time,
                "personCount": 2,
                "socialName": "",
                "requests": "",
                "purpose": "",
                "reservationMemo2": "",
                "holdingSeq": 90875464,
                "eatingTerm": 210,
                "menuSetSeq": 13388,
                "selectedMenus": [
                    {
                        "menuItemSeq": "11090",
                        "count": 2
                    }
                ],
                "onlineNoticeSeq": 0,
                "selectedTableTypes": [
                    ""
                ],
                "userSelectedTableTypes": [
                    "_ALL_"
                ],
                "deviceType": "A",
                "resp2": resp2
            },
            "paymentForm": {
                "userBillkeyRef": "d_ZRErwvOqvafv7dU5QgCg",
                "encryptedCatchPayPassword": "KrNqqppvZm5iD72glfHsRjnLR6icaZ5a8EH9VwAfscbLryowCI3MNeJYaGZxDzVil98wXl4vOOhLTUPNim9tzvtmuKpg9l3e0Cy9pxMEj424c/d4fJ1cUc4tgEyr/8En8uXG/Oqt93x2hooCnR2U74tsq3mgvNKYsZ36ZoaUmB8="
            },
            "reservationAgreementCodes": [
                "A11",
                "A10"
            ],
            "token": token,
            "shopAlias": self.shop_alias
        }
        headers = {
            'Referer':f'https://app.catchtable.co.kr/ct/reservation/form',
        }
        data = self.session.post(url="reservation/regist", headers=headers, payload=payload)
        
        return data
