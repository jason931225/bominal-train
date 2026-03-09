try:
    from curl_cffi.requests.exceptions import ConnectionError, Timeout
except ImportError:
    from requests.exceptions import ConnectionError, Timeout

from datetime import datetime, timedelta
from json.decoder import JSONDecodeError
from random import gammavariate
from termcolor import colored
from typing import Awaitable, Callable, List, Optional, Tuple, Union

import asyncio
import click
import inquirer
import keyring
import os
import telegram
import time
import re
import sys

from .ktx import (
    Korail,
    KorailError,
    ReserveOption,
    TrainType,
    AdultPassenger,
    ChildPassenger,
    SeniorPassenger,
    Disability1To3Passenger,
    Disability4To6Passenger,
)

from .srt import (
    SRT,
    SRTError,
    SRTNetFunnelError,
    SeatType,
    Adult,
    Child,
    Senior,
    Disability1To3,
    Disability4To6,
)


STATIONS = {
    "SRT": [
        "수서",
        "동탄",
        "평택지제",
        "경주",
        "곡성",
        "공주",
        "광주송정",
        "구례구",
        "김천(구미)",
        "나주",
        "남원",
        "대전",
        "동대구",
        "마산",
        "목포",
        "밀양",
        "부산",
        "서대구",
        "순천",
        "여수EXPO",
        "여천",
        "오송",
        "울산(통도사)",
        "익산",
        "전주",
        "정읍",
        "진영",
        "진주",
        "창원",
        "창원중앙",
        "천안아산",
        "포항",
    ],
    "KTX": [
        "서울",
        "용산",
        "영등포",
        "광명",
        "수원",
        "천안아산",
        "오송",
        "대전",
        "서대전",
        "김천구미",
        "동대구",
        "경주",
        "포항",
        "밀양",
        "구포",
        "부산",
        "울산(통도사)",
        "마산",
        "창원중앙",
        "경산",
        "논산",
        "익산",
        "정읍",
        "광주송정",
        "목포",
        "전주",
        "순천",
        "여수EXPO",
        "청량리",
        "강릉",
        "행신",
        "정동진",
    ],
}
DEFAULT_STATIONS = {
    "SRT": ["수서", "대전", "동대구", "부산"],
    "KTX": ["서울", "대전", "동대구", "부산"],
}

# 예약 간격 (평균 간격 (초) = SHAPE * SCALE): gamma distribution (1.25 +/- 0.25 s)
RESERVE_INTERVAL_SHAPE = 4
RESERVE_INTERVAL_SCALE = 0.25
RESERVE_INTERVAL_MIN = 0.25
TRANSIENT_NETWORK_RETRY_LIMIT = 3

WAITING_BAR = ["|", "/", "-", "\\"]

RailType = Union[str, None]
ChoiceType = Union[int, None]


def _report_keyring_error(err, action):
    print(f"키체인 {action} 중 오류가 발생했습니다: {err}")
    if "-25244" not in str(err):
        return

    current_python = sys.executable
    resolved_python = os.path.realpath(current_python)

    print("macOS 키체인 항목의 소유자/접근 권한이 현재 Python 실행 파일과 맞지 않습니다.")
    print("해결 방법 1: Keychain Access에서 SRT/KTX 관련 항목을 삭제한 뒤 다시 입력하세요.")
    print(
        f"해결 방법 2: Keychain Access > Access Control에 다음 실행 파일을 추가하세요: {current_python}"
    )
    if resolved_python != current_python:
        print(f"필요하면 실제 실행 파일도 추가하세요: {resolved_python}")


def _keyring_get(service, account, default=None):
    try:
        value = keyring.get_password(service, account)
    except keyring.errors.KeyringError as err:
        _report_keyring_error(err, "조회")
        return default
    return default if value is None else value


def _keyring_set(service, account, value):
    try:
        keyring.set_password(service, account, value)
    except keyring.errors.KeyringError as err:
        _report_keyring_error(err, "저장")
        return False
    return True


def _keyring_delete(service, account, report_error=True):
    try:
        keyring.delete_password(service, account)
    except keyring.errors.KeyringError as err:
        if report_error:
            _report_keyring_error(err, "삭제")
        return False
    return True


def _keyring_set_many(service, values, ok_key=None):
    written_accounts = []

    for account, value in values.items():
        if not _keyring_set(service, account, str(value)):
            for written_account in reversed(written_accounts):
                _keyring_delete(service, written_account, report_error=False)
            return False
        written_accounts.append(account)

    if ok_key is None:
        return True

    if _keyring_set(service, ok_key, "1"):
        return True

    for written_account in reversed(written_accounts):
        _keyring_delete(service, written_account, report_error=False)
    return False


@click.command()
@click.option("--debug", is_flag=True, help="Debug mode")
def srtgo(debug=False):
    MENU_CHOICES = [
        ("예매 시작", 1),
        ("예매 확인/결제/취소", 2),
        ("로그인 설정", 3),
        ("텔레그램 설정", 4),
        ("카드 설정", 5),
        ("역 설정", 6),
        ("역 직접 수정", 7),
        ("예매 옵션 설정", 8),
        ("나가기", -1),
    ]

    RAIL_CHOICES = [
        (colored("SRT", "red"), "SRT"),
        (colored("KTX", "cyan"), "KTX"),
        ("취소", -1),
    ]

    ACTIONS = {
        1: lambda rt: reserve(rt, debug),
        2: lambda rt: check_reservation(rt, debug),
        3: lambda rt: set_login(rt, debug),
        4: lambda _: set_telegram(),
        5: lambda _: set_card(),
        6: lambda rt: set_station(rt),
        7: lambda rt: edit_station(rt),
        8: lambda _: set_options(),
    }

    while True:
        choice = inquirer.list_input(
            message="메뉴 선택 (↕:이동, Enter: 선택)", choices=MENU_CHOICES
        )

        if choice == -1:
            break

        if choice in {1, 2, 3, 6, 7}:
            rail_type = inquirer.list_input(
                message="열차 선택 (↕:이동, Enter: 선택, Ctrl-C: 취소)",
                choices=RAIL_CHOICES,
            )
            if rail_type in {-1, None}:
                continue
        else:
            rail_type = None

        action = ACTIONS.get(choice)
        if action:
            action(rail_type)


def set_station(rail_type: RailType) -> bool:
    stations, default_station_key = get_station(rail_type)

    if not (
        station_info := inquirer.prompt(
            [
                inquirer.Checkbox(
                    "stations",
                    message="역 선택 (↕:이동, Space: 선택, Enter: 완료, Ctrl-A: 전체선택, Ctrl-R: 선택해제, Ctrl-C: 취소)",
                    choices=stations,
                    default=default_station_key,
                )
            ]
        )
    ):
        return False

    if not (selected := station_info["stations"]):
        print("선택된 역이 없습니다.")
        return False

    if not _keyring_set(
        rail_type, "station", (selected_stations := ",".join(selected))
    ):
        return False
    print(f"선택된 역: {selected_stations}")
    return True


def edit_station(rail_type: RailType) -> bool:
    stations, default_station_key = get_station(rail_type)
    station_info = inquirer.prompt(
        [
            inquirer.Text(
                "stations",
                message="역 수정 (예: 수서,대전,동대구)",
                default=_keyring_get(rail_type, "station", "") or "",
            )
        ]
    )
    if not station_info:
        return False

    if not (selected := station_info["stations"]):
        print("선택된 역이 없습니다.")
        return False

    selected = [s.strip() for s in selected.split(",")]

    # Verify all stations contain Korean characters
    hangul = re.compile("[가-힣]+")
    for station in selected:
        if not hangul.search(station):
            print(f"'{station}'는 잘못된 입력입니다. 기본 역으로 설정합니다.")
            selected = DEFAULT_STATIONS[rail_type]
            break

    if not _keyring_set(
        rail_type, "station", (selected_stations := ",".join(selected))
    ):
        return False
    print(f"선택된 역: {selected_stations}")
    return True


def get_station(rail_type: RailType) -> Tuple[List[str], List[int]]:
    stations = STATIONS[rail_type]
    station_key = _keyring_get(rail_type, "station")

    if not station_key:
        return stations, DEFAULT_STATIONS[rail_type]

    valid_keys = [x for x in station_key.split(",")]
    return stations, valid_keys


def set_options():
    default_options = get_options()
    choices = inquirer.prompt(
        [
            inquirer.Checkbox(
                "options",
                message="예매 옵션 선택 (Space: 선택, Enter: 완료, Ctrl-A: 전체선택, Ctrl-R: 선택해제, Ctrl-C: 취소)",
                choices=[
                    ("어린이", "child"),
                    ("경로우대", "senior"),
                    ("중증장애인", "disability1to3"),
                    ("경증장애인", "disability4to6"),
                    ("KTX만", "ktx"),
                ],
                default=default_options,
            )
        ]
    )

    if choices is None:
        return

    options = choices.get("options", [])
    _keyring_set("SRT", "options", ",".join(options))


def get_options():
    options = _keyring_get("SRT", "options", "") or ""
    return options.split(",") if options else []


def set_telegram() -> bool:
    token = _keyring_get("telegram", "token", "") or ""
    chat_id = _keyring_get("telegram", "chat_id", "") or ""

    telegram_info = inquirer.prompt(
        [
            inquirer.Text(
                "token",
                message="텔레그램 token (Enter: 완료, Ctrl-C: 취소)",
                default=token,
            ),
            inquirer.Text(
                "chat_id",
                message="텔레그램 chat_id (Enter: 완료, Ctrl-C: 취소)",
                default=chat_id,
            ),
        ]
    )
    if not telegram_info:
        return False

    token, chat_id = telegram_info["token"], telegram_info["chat_id"]

    if not _keyring_set_many(
        "telegram", {"token": token, "chat_id": chat_id}, ok_key="ok"
    ):
        return False

    try:
        tgprintf = get_telegram()
        asyncio.run(tgprintf("[SRTGO] 텔레그램 설정 완료"))
        return True
    except Exception as err:
        print(err)
        _keyring_delete("telegram", "ok", report_error=False)
        return False


def get_telegram() -> Optional[Callable[[str], Awaitable[None]]]:
    token = _keyring_get("telegram", "token")
    chat_id = _keyring_get("telegram", "chat_id")

    async def tgprintf(text):
        if token and chat_id:
            bot = telegram.Bot(token=token)
            async with bot:
                await bot.send_message(chat_id=chat_id, text=text)

    return tgprintf


def set_card() -> None:
    card_info = {
        "number": _keyring_get("card", "number", "") or "",
        "password": _keyring_get("card", "password", "") or "",
        "birthday": _keyring_get("card", "birthday", "") or "",
        "expire": _keyring_get("card", "expire", "") or "",
    }

    card_info = inquirer.prompt(
        [
            inquirer.Password(
                "number",
                message="신용카드 번호 (하이픈 제외(-), Enter: 완료, Ctrl-C: 취소)",
                default=card_info["number"],
            ),
            inquirer.Password(
                "password",
                message="카드 비밀번호 앞 2자리 (Enter: 완료, Ctrl-C: 취소)",
                default=card_info["password"],
            ),
            inquirer.Password(
                "birthday",
                message="생년월일 (YYMMDD) / 사업자등록번호 (Enter: 완료, Ctrl-C: 취소)",
                default=card_info["birthday"],
            ),
            inquirer.Password(
                "expire",
                message="카드 유효기간 (YYMM, Enter: 완료, Ctrl-C: 취소)",
                default=card_info["expire"],
            ),
        ]
    )
    if card_info:
        _keyring_set_many("card", card_info, ok_key="ok")


def pay_card(rail, reservation) -> bool:
    if _keyring_get("card", "ok"):
        birthday = _keyring_get("card", "birthday")
        return rail.pay_with_card(
            reservation,
            _keyring_get("card", "number"),
            _keyring_get("card", "password"),
            birthday,
            _keyring_get("card", "expire"),
            0,
            "J" if len(birthday) == 6 else "S",
        )
    return False


def set_login(rail_type="SRT", debug=False):
    credentials = {
        "id": _keyring_get(rail_type, "id", "") or "",
        "pass": _keyring_get(rail_type, "pass", "") or "",
    }

    login_info = inquirer.prompt(
        [
            inquirer.Text(
                "id",
                message=f"{rail_type} 계정 아이디 (멤버십 번호, 이메일, 전화번호)",
                default=credentials["id"],
            ),
            inquirer.Password(
                "pass",
                message=f"{rail_type} 계정 패스워드",
                default=credentials["pass"],
            ),
        ]
    )
    if not login_info:
        return False

    try:
        rail = SRT(
            login_info["id"], login_info["pass"], verbose=debug
        ) if rail_type == "SRT" else Korail(
            login_info["id"], login_info["pass"], verbose=debug
        )

        return (
            rail
            if _keyring_set_many(
                rail_type,
                {"id": login_info["id"], "pass": login_info["pass"]},
                ok_key="ok",
            )
            else False
        )
    except SRTError as err:
        print(err)
        _keyring_delete(rail_type, "ok", report_error=False)
        return False


def login(rail_type="SRT", debug=False):
    if (
        _keyring_get(rail_type, "id") is None
        or _keyring_get(rail_type, "pass") is None
    ):
        if not (rail := set_login(rail_type, debug=debug)):
            return None
        return rail

    user_id = _keyring_get(rail_type, "id")
    password = _keyring_get(rail_type, "pass")
    if user_id is None or password is None:
        return None

    rail = SRT if rail_type == "SRT" else Korail
    return rail(user_id, password, verbose=debug)


def reserve(rail_type="SRT", debug=False):
    rail = login(rail_type, debug=debug)
    if rail is None:
        return
    is_srt = rail_type == "SRT"

    # Get date, time, stations, and passenger info
    now = datetime.now() + timedelta(minutes=10)
    today = now.strftime("%Y%m%d")
    this_time = now.strftime("%H%M%S")

    defaults = {
        "departure": _keyring_get(
            rail_type, "departure", "수서" if is_srt else "서울"
        ),
        "arrival": _keyring_get(rail_type, "arrival", "동대구"),
        "date": _keyring_get(rail_type, "date", today),
        "time": _keyring_get(rail_type, "time", "120000"),
        "adult": int(_keyring_get(rail_type, "adult", 1)),
        "child": int(_keyring_get(rail_type, "child", 0)),
        "senior": int(_keyring_get(rail_type, "senior", 0)),
        "disability1to3": int(_keyring_get(rail_type, "disability1to3", 0)),
        "disability4to6": int(_keyring_get(rail_type, "disability4to6", 0)),
    }

    # Set default stations if departure equals arrival
    if defaults["departure"] == defaults["arrival"]:
        defaults["arrival"] = (
            "동대구" if defaults["departure"] in ("수서", "서울") else None
        )
        defaults["departure"] = (
            defaults["departure"]
            if defaults["arrival"]
            else ("수서" if is_srt else "서울")
        )

    stations, station_key = get_station(rail_type)
    options = get_options()

    # Calculate dynamic booking window (SRT: D-30, KTX: D-31; both open at 07:00)
    if is_srt:
        max_days = 30 if now.hour >= 7 else 29
    else:
        max_days = 31 if now.hour >= 7 else 30

    # Generate date choices within the window
    date_choices = [
        (
            (now + timedelta(days=i)).strftime("%Y/%m/%d %a"),
            (now + timedelta(days=i)).strftime("%Y%m%d"),
        )
        for i in range(max_days + 1)
    ]
    time_choices = [(f"{h:02d}", f"{h:02d}0000") for h in range(24)]

    # Build inquirer questions
    q_info = [
        inquirer.List(
            "departure",
            message="출발역 선택 (↕:이동, Enter: 선택, Ctrl-C: 취소)",
            choices=station_key,
            default=defaults["departure"],
        ),
        inquirer.List(
            "arrival",
            message="도착역 선택 (↕:이동, Enter: 선택, Ctrl-C: 취소)",
            choices=station_key,
            default=defaults["arrival"],
        ),
        inquirer.List(
            "date",
            message="출발 날짜 선택 (↕:이동, Enter: 선택, Ctrl-C: 취소)",
            choices=date_choices,
            default=defaults["date"],
        ),
        inquirer.List(
            "time",
            message="출발 시각 선택 (↕:이동, Enter: 선택, Ctrl-C: 취소)",
            choices=time_choices,
            default=defaults["time"],
        ),
        inquirer.List(
            "adult",
            message="성인 승객수 (↕:이동, Enter: 선택, Ctrl-C: 취소)",
            choices=range(10),
            default=defaults["adult"],
        ),
    ]

    passenger_types = {
        "child": "어린이",
        "senior": "경로우대",
        "disability1to3": "1~3급 장애인",
        "disability4to6": "4~6급 장애인",
    }

    passenger_classes = {
        "adult": Adult if is_srt else AdultPassenger,
        "child": Child if is_srt else ChildPassenger,
        "senior": Senior if is_srt else SeniorPassenger,
        "disability1to3": Disability1To3 if is_srt else Disability1To3Passenger,
        "disability4to6": Disability4To6 if is_srt else Disability4To6Passenger,
    }

    PASSENGER_TYPE = {
        passenger_classes["adult"]: "어른/청소년",
        passenger_classes["child"]: "어린이",
        passenger_classes["senior"]: "경로우대",
        passenger_classes["disability1to3"]: "1~3급 장애인",
        passenger_classes["disability4to6"]: "4~6급 장애인",
    }

    # Add passenger type questions if enabled in options
    for key, label in passenger_types.items():
        if key in options:
            q_info.append(
                inquirer.List(
                    key,
                    message=f"{label} 승객수 (↕:이동, Enter: 선택, Ctrl-C: 취소)",
                    choices=range(10),
                    default=defaults[key],
                )
            )

    info = inquirer.prompt(q_info)

    # Validate input info
    if not info:
        print(colored("예매 정보 입력 중 취소되었습니다", "green", "on_red") + "\n")
        return

    if info["departure"] == info["arrival"]:
        print(colored("출발역과 도착역이 같습니다", "green", "on_red") + "\n")
        return

    # Save preferences
    _keyring_set_many(rail_type, {key: str(value) for key, value in info.items()})

    # Adjust time if needed
    if info["date"] == today and int(info["time"]) < int(this_time):
        info["time"] = this_time

    # Build passenger list
    passengers = []
    total_count = 0
    for key, cls in passenger_classes.items():
        if key in info and info[key] > 0:
            passengers.append(cls(info[key]))
            total_count += info[key]

    # Validate passenger count
    if not passengers:
        print(colored("승객수는 0이 될 수 없습니다", "green", "on_red") + "\n")
        return

    if total_count >= 10:
        print(colored("승객수는 10명을 초과할 수 없습니다", "green", "on_red") + "\n")
        return

    msg_passengers = [
        f"{PASSENGER_TYPE[type(passenger)]} {passenger.count}명"
        for passenger in passengers
    ]
    print(*msg_passengers)

    # Search for trains
    params = {
        "dep": info["departure"],
        "arr": info["arrival"],
        "date": info["date"],
        "time": info["time"],
        "passengers": [passenger_classes["adult"](total_count)],
        **(
            {"available_only": False}
            if is_srt
            else {
                "include_no_seats": True,
                **({"train_type": TrainType.KTX} if "ktx" in options else {}),
            }
        ),
    }

    trains = rail.search_train(**params)

    def train_decorator(train):
        msg = train.__repr__()
        return (
            msg.replace("예약가능", colored("가능", "green"))
            .replace("가능", colored("가능", "green"))
            .replace("신청하기", colored("가능", "green"))
        )

    if not trains:
        print(colored("예약 가능한 열차가 없습니다", "green", "on_red") + "\n")
        return

    # Get train selection
    q_choice = [
        inquirer.Checkbox(
            "trains",
            message="예약할 열차 선택 (↕:이동, Space: 선택, Enter: 완료, Ctrl-A: 전체선택, Ctrl-R: 선택해제, Ctrl-C: 취소)",
            choices=[(train_decorator(train), i) for i, train in enumerate(trains)],
            default=None,
        ),
    ]

    choice = inquirer.prompt(q_choice)
    if choice is None or not choice["trains"]:
        print(colored("선택한 열차가 없습니다!", "green", "on_red") + "\n")
        return

    n_trains = len(choice["trains"])

    # Get seat type preference
    seat_type = SeatType if is_srt else ReserveOption
    q_options = [
        inquirer.List(
            "type",
            message="선택 유형",
            choices=[
                ("일반실 우선", seat_type.GENERAL_FIRST),
                ("일반실만", seat_type.GENERAL_ONLY),
                ("특실 우선", seat_type.SPECIAL_FIRST),
                ("특실만", seat_type.SPECIAL_ONLY),
            ],
        ),
        inquirer.Confirm("pay", message="예매 시 카드 결제", default=False),
    ]

    options = inquirer.prompt(q_options)
    if options is None:
        print(colored("예매 정보 입력 중 취소되었습니다", "green", "on_red") + "\n")
        return

    # Reserve function
    def _reserve(train):
        reserve = rail.reserve(train, passengers=passengers, option=options["type"])
        msg = f"{reserve}"
        if hasattr(reserve, "tickets") and reserve.tickets:
            msg += "\n" + "\n".join(map(str, reserve.tickets))

        print(colored(f"\n\n🎫 🎉 예매 성공!!! 🎉 🎫\n{msg}\n", "red", "on_green"))

        if options["pay"] and not reserve.is_waiting and pay_card(rail, reserve):
            print(
                colored("\n\n💳 ✨ 결제 성공!!! ✨ 💳\n\n", "green", "on_red"), end=""
            )
            msg += "\n결제 완료"

        tgprintf = get_telegram()
        asyncio.run(tgprintf(msg))

    # Reservation loop
    i_try = 0
    transient_network_errors = 0
    start_time = time.time()
    while True:
        try:
            i_try += 1
            elapsed_time = time.time() - start_time
            hours, remainder = divmod(int(elapsed_time), 3600)
            minutes, seconds = divmod(remainder, 60)
            print(
                f"\r예매 대기 중... {WAITING_BAR[i_try & 3]} {i_try:4d} ({hours:02d}:{minutes:02d}:{seconds:02d}) ",
                end="",
                flush=True,
            )

            trains = rail.search_train(**params)
            transient_network_errors = 0
            for i in choice["trains"]:
                if _is_seat_available(trains[i], options["type"], rail_type):
                    _reserve(trains[i])
                    return
            _sleep()

        except KeyboardInterrupt:
            print("\n예매를 취소했습니다.")
            return

        except SRTError as ex:
            transient_network_errors = 0
            msg = ex.msg
            if "정상적인 경로로 접근 부탁드립니다" in msg or isinstance(
                ex, SRTNetFunnelError
            ):
                if debug:
                    print(
                        f"\nException: {ex}\nType: {type(ex)}\nArgs: {ex.args}\nMessage: {msg}"
                    )
                rail.clear()
            elif "로그인 후 사용하십시오" in msg:
                if debug:
                    print(
                        f"\nException: {ex}\nType: {type(ex)}\nArgs: {ex.args}\nMessage: {msg}"
                    )
                rail = login(rail_type, debug=debug)
                if rail is None:
                    return
                if not rail.is_login and not _handle_error(ex):
                    return
            elif not any(
                err in msg
                for err in (
                    "잔여석없음",
                    "사용자가 많아 접속이 원활하지 않습니다",
                    "예약대기 접수가 마감되었습니다",
                    "예약대기자한도수초과",
                )
            ):
                if not _handle_error(ex):
                    return
            _sleep()

        except KorailError as ex:
            transient_network_errors = 0
            msg = ex.msg
            if "Need to Login" in msg:
                rail = login(rail_type, debug=debug)
                if rail is None:
                    return
                if not rail.is_login and not _handle_error(ex):
                    return
            elif not any(
                err in msg
                for err in ("Sold out", "잔여석없음", "예약대기자한도수초과")
            ):
                if not _handle_error(ex):
                    return
            _sleep()

        except JSONDecodeError as ex:
            transient_network_errors = 0
            if debug:
                print(
                    f"\nException: {ex}\nType: {type(ex)}\nArgs: {ex.args}\nMessage: {ex.msg}"
                )
            _sleep()
            rail = login(rail_type, debug=debug)
            if rail is None:
                return

        except (ConnectionError, Timeout) as ex:
            transient_network_errors += 1
            if transient_network_errors <= TRANSIENT_NETWORK_RETRY_LIMIT:
                _handle_transient_network_error(ex, debug=debug)
                _sleep()
                continue

            rail = login(rail_type, debug=debug)
            if rail is None:
                return

            transient_network_errors = 0
            if not _handle_error(ex, "네트워크 연결이 계속 불안정합니다. 다시 로그인했습니다."):
                return

        except Exception as ex:
            if _is_user_cancelled_exception(ex):
                print("\n예매를 취소했습니다.")
                return
            if debug:
                print("\nUndefined exception")
            if not _handle_error(ex):
                return
            rail = login(rail_type, debug=debug)
            if rail is None:
                return


def _sleep():
    time.sleep(
        gammavariate(RESERVE_INTERVAL_SHAPE, RESERVE_INTERVAL_SCALE)
        + RESERVE_INTERVAL_MIN
    )


def _handle_transient_network_error(ex, debug=False):
    if debug:
        print(
            f"\nException: {ex}\nType: {type(ex)}\nArgs: {ex.args}\nMessage: {getattr(ex, 'msg', 'No message attribute')}"
        )
    print("\n네트워크 연결이 불안정합니다. 재시도합니다.")


def _is_user_cancelled_exception(ex) -> bool:
    if isinstance(ex, KeyboardInterrupt):
        return True

    for chained in (getattr(ex, "__cause__", None), getattr(ex, "__context__", None)):
        if isinstance(chained, KeyboardInterrupt):
            return True

    message = str(ex)
    return "curl: (23)" in message and "Failure writing output to destination" in message


def _handle_error(ex, msg=None):
    msg = (
        msg
        or f"\nException: {ex}, Type: {type(ex)}, Message: {ex.msg if hasattr(ex, 'msg') else 'No message attribute'}"
    )
    print(msg)
    tgprintf = get_telegram()
    asyncio.run(tgprintf(msg))
    return inquirer.confirm(message="계속하시겠습니까?", default=True)


def _is_seat_available(train, seat_type, rail_type):
    if rail_type == "SRT":
        if not train.seat_available():
            return train.reserve_standby_available()
        if seat_type in [SeatType.GENERAL_FIRST, SeatType.SPECIAL_FIRST]:
            return train.seat_available()
        if seat_type == SeatType.GENERAL_ONLY:
            return train.general_seat_available()
        return train.special_seat_available()
    else:
        if not train.has_seat():
            return train.has_waiting_list()
        if seat_type in [ReserveOption.GENERAL_FIRST, ReserveOption.SPECIAL_FIRST]:
            return train.has_seat()
        if seat_type == ReserveOption.GENERAL_ONLY:
            return train.has_general_seat()
        return train.has_special_seat()


def check_reservation(rail_type="SRT", debug=False):
    rail = login(rail_type, debug=debug)
    if rail is None:
        return

    while True:
        reservations = (
            rail.get_reservations() if rail_type == "SRT" else rail.reservations()
        )
        tickets = [] if rail_type == "SRT" else rail.tickets()

        all_reservations = []
        for t in tickets:
            t.is_ticket = True
            all_reservations.append(t)
        for r in reservations:
            if hasattr(r, "paid") and r.paid:
                r.is_ticket = True
            else:
                r.is_ticket = False
            all_reservations.append(r)

        if not reservations and not tickets:
            print(colored("예약 내역이 없습니다", "green", "on_red") + "\n")
            return

        choices = [
            (str(reservation), i) for i, reservation in enumerate(all_reservations)
        ] + [("텔레그램으로 예매 정보 전송", -2), ("돌아가기", -1)]

        choice = inquirer.list_input(message="예약 취소 (Enter: 결정)", choices=choices)

        # No choice or go back
        if choice in (None, -1):
            return

        # Send reservation info to telegram
        if choice == -2:
            out = []
            if all_reservations:
                out.append("[ 예매 내역 ]")
                for reservation in all_reservations:
                    out.append(f"🚅{reservation}")
                    if rail_type == "SRT":
                        out.extend(map(str, reservation.tickets))

            if out:
                tgprintf = get_telegram()
                asyncio.run(tgprintf("\n".join(out)))
            return

        # If choice is an unpaid reservation, ask to pay or cancel
        if (
            not all_reservations[choice].is_ticket
            and not all_reservations[choice].is_waiting
        ):
            answer = inquirer.list_input(
                message=f"결재 대기 승차권: {all_reservations[choice]}",
                choices=[("결제하기", 1), ("취소하기", 2)],
            )

            if answer == 1:
                if pay_card(rail, all_reservations[choice]):
                    print(
                        colored("\n\n💳 ✨ 결제 성공!!! ✨ 💳\n\n", "green", "on_red"),
                        end="",
                    )
            elif answer == 2:
                rail.cancel(all_reservations[choice])
            return

        # Else
        if inquirer.confirm(
            message=colored("정말 취소하시겠습니까", "green", "on_red")
        ):
            try:
                if all_reservations[choice].is_ticket:
                    rail.refund(all_reservations[choice])
                else:
                    rail.cancel(all_reservations[choice])
            except Exception as err:
                raise err
            return


if __name__ == "__main__":
    srtgo()
