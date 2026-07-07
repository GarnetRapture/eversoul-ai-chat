<p align="right">
  <img src="https://flagcdn.com/20x15/kr.png" width="20" height="15" alt="KR" /> <strong>한국어</strong> &nbsp;|&nbsp;
  <a href="README.en.md"><img src="https://flagcdn.com/20x15/us.png" width="20" height="15" alt="US" /> English</a> &nbsp;|&nbsp;
  <a href="README.zh-CN.md"><img src="https://flagcdn.com/20x15/cn.png" width="20" height="15" alt="CN" /> 简体中文</a>
</p>

<p align="center">
  <img src="public/eversoul-assets/backgrounds/talk/Talk_BG_Castle_Aurelia.png" width="960" alt="EverSoul AI Chat Banner" />
</p>

<h1 align="center">EverSoul AI Chat</h1>
<p align="center"><i>완전한 로컬 구동 AI 채팅 클라이언트</i></p>

<p align="center">
  <img src="https://img.shields.io/badge/version-0.0.14-blue?style=flat-square" alt="Version" />
  <img src="https://img.shields.io/badge/license-Apache_2.0-green?style=flat-square" alt="License" />
  <img src="https://img.shields.io/badge/Tauri-2-FFC107?style=flat-square&logo=tauri" alt="Tauri" />
  <img src="https://img.shields.io/badge/React-19.1-61DAFB?style=flat-square&logo=react" alt="React" />
  <img src="https://img.shields.io/badge/Rust-2021_edition-000000?style=flat-square&logo=rust" alt="Rust" />
  <img src="https://img.shields.io/badge/SQLite-bundled-003B57?style=flat-square&logo=sqlite" alt="SQLite" />
  <img src="https://img.shields.io/badge/spirits-95-9b5de5?style=flat-square" alt="Spirits" />
  <img src="https://img.shields.io/badge/talk_backgrounds-522-f15bb5?style=flat-square" alt="Backgrounds" />
  <img src="https://img.shields.io/badge/languages-ko%20%7C%20en%20%7C%20zh__tw%20%7C%20zh__cn-00bbf9?style=flat-square" alt="Languages" />
</p>

---

## 🌟 개요

**EverSoul AI Chat**은 에버소울을 간직할 새로운 로컬 AI 채팅 프로젝트입니다. 정령들의 기억을 보존한다는 의미로 만들었습니다. 에버소울에 등장하는 정령 95명을 실제 게임 데이터 그대로 불러와, 각자의 성격과 말투로 자유롭게 대화할 수 있습니다.

대화를 만들어내는 AI는 전부 내 컴퓨터 안에서만 돌아갑니다. 어떤 대화 내용도 밖으로 나가지 않고, 정령과 나눈 이야기는 전부 내 컴퓨터 안에서만 오갑니다.

그래서 정령 95명 전원의 실제 게임 그림, 대화 배경 522장, 에버톡 화면에서 쓰던 UI까지 프로젝트 안에 그대로 담아뒀습니다. 각 정령의 이름과 성격, 말투는 `data/personas/`에 정령마다 하나씩 정리되어 있고, 한국어·영어·중국어(번체/간체) 네 개 언어로 미리 준비해 두었기 때문에 언어를 바꿔도 그 정령다움은 그대로 유지됩니다.

<p align="center">
  <img src="public/eversoul-assets/spirits/GarnetRapture/base/GarnetRapture_1024.png" width="120" alt="GarnetRapture" />
  <img src="public/eversoul-assets/spirits/Adrianne/base/Adrianne_1024.png" width="120" alt="Adrianne" />
  <img src="public/eversoul-assets/spirits/Naomi/base/Naomi_1024.png" width="120" alt="Naomi" />
  <img src="public/eversoul-assets/spirits/Laura/base/Laura_1024.png" width="120" alt="Laura" />
  <img src="public/eversoul-assets/spirits/Weiss/base/Weiss_1024.png" width="120" alt="Weiss" />
  <img src="public/eversoul-assets/spirits/Lilith/base/Lilith_1024.png" width="120" alt="Lilith" />
</p>

---

## 🎨 전체 정령 갤러리 (95종)

`data/personas/*.json` 95개 파일을 전부 살펴서 정령 그림과 한국어(ko)·영어(en)·중국어 간체(zh_cn) 이름을 실제 데이터 그대로 나열한 도감입니다. 그림 폴더 이름은 `src/domains/persona/logic.ts`의 `resolveSpiritAssetFolder`가 찾는 방식 그대로 가져왔습니다(게임 내 표시명과 실제 그림 폴더명이 다른 26명은 `explicitAssetFolders` 매핑을 그대로 따랐습니다).

<table>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/Oyome/base/Oyome_1024.png" width="64"/><br/><sub>아야메<br/>Ayame<br/>綾織</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/AyameTsukuyomi/base/AyameTsukuyomi_1024.png" width="64"/><br/><sub>아야메(츠쿠요미)<br/>Ayame (Tsukuyomi)<br/>綾織（月讀）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Aki/base/Aki_1024.png" width="64"/><br/><sub>아키<br/>Aki<br/>秋</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Alisha/base/Alisha_1024.png" width="64"/><br/><sub>알리샤<br/>Alisha<br/>艾麗西雅</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Adrianne/base/Adrianne_1024.png" width="64"/><br/><sub>아드리안<br/>Adrianne<br/>阿德里安</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Aira/base/Aira_1024.png" width="64"/><br/><sub>아이라<br/>Aira<br/>艾拉</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/ClaudiaArchangel/base/ClaudiaArchangel_1024.png" width="64"/><br/><sub>클라우디아(대천사)<br/>Claudia (Archangel)<br/>克勞迪婭（大天使）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Beatrice/base/Beatrice_1024.png" width="64"/><br/><sub>클레르<br/>Claire<br/>克萊兒</sub></td>
</tr>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/Catarina/base/Catarina_1024.png" width="64"/><br/><sub>셰리<br/>Cherrie<br/>雪莉</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Chloe/base/Chloe_1024.png" width="64"/><br/><sub>클로이<br/>Chloe<br/>克羅伊</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/CherrieRoman/base/CherrieRoman_1024.png" width="64"/><br/><sub>셰리(낭만)<br/>Cherrie (Romantic)<br/>雪莉（浪漫）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Clara/base/Clara_1024.png" width="64"/><br/><sub>클라라<br/>Clara<br/>克拉拉</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Claudia/base/Claudia_1024.png" width="64"/><br/><sub>클라우디아<br/>Claudia<br/>克勞迪婭</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Olivia/base/Olivia_1024.png" width="64"/><br/><sub>가넷<br/>Garnet<br/>佳妮特</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/CatherineBrave/base/CatherineBrave_1024.png" width="64"/><br/><sub>캐서린(광휘)<br/>Catherine (Radiance)<br/>凱瑟琳（光輝）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Dominique/base/Dominique_1024.png" width="64"/><br/><sub>도미니크<br/>Dominique<br/>多米尼克</sub></td>
</tr>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/Eileen/base/Eileen_1024.png" width="64"/><br/><sub>에일린<br/>Eileen<br/>艾琳</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Ina/base/Ina_1024.png" width="64"/><br/><sub>이나<br/>Ina<br/>伊娜</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Hazel/base/Hazel_1024.png" width="64"/><br/><sub>헤이즐<br/>Hazel<br/>黑伊茲爾</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Catherine/base/Catherine_1024.png" width="64"/><br/><sub>캐서린<br/>Catherine<br/>凱瑟琳</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Dora/base/Dora_1024.png" width="64"/><br/><sub>도라<br/>Dora<br/>朵菈</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/GarnetRapture/base/GarnetRapture_1024.png" width="64"/><br/><sub>가넷(열락)<br/>Garnet (Rapture)<br/>佳妮特（狂喜）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Honglan/base/Honglan_1024.png" width="64"/><br/><sub>홍란<br/>Honglan<br/>紅蘭</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Hanul/base/Hanul_1024.png" width="64"/><br/><sub>한울<br/>Hanul<br/>韓羽</sub></td>
</tr>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/Edith/base/Edith_1024.png" width="64"/><br/><sub>이디스<br/>Edith<br/>伊迪絲</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Milia/base/Milia_1024.png" width="64"/><br/><sub>플린<br/>Flynn<br/>弗林</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Erusha/base/Erusha_1024.png" width="64"/><br/><sub>에루샤<br/>Erusha<br/>艾魯莎</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/HonglanCombat/base/HonglanCombat_1024.png" width="64"/><br/><sub>홍란(무쌍)<br/>Honglan (Peerless)<br/>紅蘭（無雙）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Erika/base/Erika_1024.png" width="64"/><br/><sub>에리카<br/>Erika<br/>艾麗卡</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/HaruKamuy/base/HaruKamuy_1024.png" width="64"/><br/><sub>하루(카무이)<br/>Haru (Kamuy)<br/>河路（神威）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Carnelian/base/Carnelian_1024.png" width="64"/><br/><sub>카넬리안<br/>Carnelian<br/>卡內莉安</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Karen/base/Karen_1024.png" width="64"/><br/><sub>카렌<br/>Karen<br/>卡倫</sub></td>
</tr>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/Joanne/base/Joanne_1024.png" width="64"/><br/><sub>조앤<br/>Joanne<br/>瓊</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Daphne/base/Daphne_1024.png" width="64"/><br/><sub>다프네<br/>Daphne<br/>達芙妮</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Eve/base/Eve_1024.png" width="64"/><br/><sub>이브<br/>Eve<br/>夏娃</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Jade/base/Jade_1024.png" width="64"/><br/><sub>제이드<br/>Jade<br/>潔依德</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Tokisaki/base/Tokisaki_1024.png" width="64"/><br/><sub>토키사키 쿠루미<br/>Kurumi Tokisaki<br/>時崎狂三</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Jacqueline/base/Jacqueline_1024.png" width="64"/><br/><sub>재클린<br/>Jacqueline<br/>潔克琳</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Larimar/base/Larimar_1024.png" width="64"/><br/><sub>라리마<br/>Larimar<br/>拉利瑪</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Mia/base/Mia_1024.png" width="64"/><br/><sub>하루<br/>Haru<br/>河路</sub></td>
</tr>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/Jiho/base/Jiho_1024.png" width="64"/><br/><sub>지호<br/>Jiho<br/>智河</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Lewayne/base/Lewayne_1024.png" width="64"/><br/><sub>르웨인<br/>Lewayne<br/>樂溫</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Blyce/base/Blyce_1024.png" width="64"/><br/><sub>브라이스<br/>Bryce<br/>布萊斯</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Kanna/base/Kanna_1024.png" width="64"/><br/><sub>칸나<br/>Kanna<br/>坎納</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/JihoMir/base/JihoMir_1024.png" width="64"/><br/><sub>지호(미르)<br/>Jiho (Mir)<br/>智河（米爾）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Beleth/base/Beleth_1024.png" width="64"/><br/><sub>벨레드<br/>Beleth<br/>貝萊德</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Linzy/base/Linzy_1024.png" width="64"/><br/><sub>린지<br/>Linzy<br/>琳賽</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Laura/base/Laura_1024.png" width="64"/><br/><sub>라우라<br/>Laura<br/>蘿拉</sub></td>
</tr>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/LinzyThanatos/base/LinzyThanatos_1024.png" width="64"/><br/><sub>린지(타나토스)<br/>Linzy (Thanatos)<br/>琳賽（桑納托斯）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Lilith/base/Lilith_1024.png" width="64"/><br/><sub>릴리트<br/>Lilith<br/>莉莉絲</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Lizelotte/base/Lizelotte_1024.png" width="64"/><br/><sub>리젤로테<br/>Lizelotte<br/>莉澤洛特</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Lute/base/Lute_1024.png" width="64"/><br/><sub>루테<br/>Lute<br/>魯特</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Manon/base/Manon_1024.png" width="64"/><br/><sub>마농<br/>Manon<br/>瑪儂</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Melfice/base/Melfice_1024.png" width="64"/><br/><sub>멜피스<br/>Melfice<br/>梅爾菲斯</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Mephisto/base/Mephisto_1024.png" width="64"/><br/><sub>메피스토펠레스<br/>Mephistopheles<br/>梅菲斯托佩萊斯</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Meryl/base/Meryl_1024.png" width="64"/><br/><sub>메릴<br/>Meryl<br/>梅莉兒</sub></td>
</tr>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/Mica/base/Mica_1024.png" width="64"/><br/><sub>미카<br/>Mica<br/>米卡</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/MephistoDawn/base/MephistoDawn_1024.png" width="64"/><br/><sub>메피스토펠레스(여명)<br/>Mephistopheles (Dawn)<br/>梅菲斯托佩萊斯（黎明）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Miriam/base/Miriam_1024.png" width="64"/><br/><sub>미리암<br/>Miriam<br/>米里昂</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Nameless/base/Nameless_1024.png" width="64"/><br/><sub>무명<br/>Nameless<br/>無名</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Nyah/base/Nyah_1024.png" width="64"/><br/><sub>나이아<br/>Naiah<br/>娜伊雅</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Naomi/base/Naomi_1024.png" width="64"/><br/><sub>나오미<br/>Naomi<br/>直美</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/MiriamMirage/base/MiriamMirage_1024.png" width="64"/><br/><sub>미리암(잔영)<br/>Miriam (Afterimage)<br/>米里昂（殘影）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Nicole/base/Nicole_1024.png" width="64"/><br/><sub>니콜<br/>Nicole<br/>妮可</sub></td>
</tr>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/Nia/base/Nia_1024.png" width="64"/><br/><sub>니아<br/>Nia<br/>妮亞</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Onyx/base/Onyx_1024.png" width="64"/><br/><sub>오닉스<br/>Onyx<br/>歐妮絲</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Nini/base/Nini_1024.png" width="64"/><br/><sub>니니<br/>Nini<br/>妮妮</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Otoha/base/Otoha_1024.png" width="64"/><br/><sub>오토하<br/>Otoha<br/>乙葉</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/PetraAwaken/base/PetraAwaken_1024.png" width="64"/><br/><sub>페트라(각혼)<br/>Petra (Awakened Soul)<br/>佩特拉（覺魂）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Rebecca/base/Rebecca_1024.png" width="64"/><br/><sub>레베카<br/>Rebecca<br/>瑞貝卡</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Rose/base/Rose_1024.png" width="64"/><br/><sub>로제<br/>Rose<br/>蘿絲</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Leah/base/Leah_1024.png" width="64"/><br/><sub>르네<br/>Renee<br/>勒內</sub></td>
</tr>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/Rita/base/Rita_1024.png" width="64"/><br/><sub>리타<br/>Rita<br/>麗塔</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/RoseCrimson/base/RoseCrimson_1024.png" width="64"/><br/><sub>로제(홍염)<br/>Rose (Prominence)<br/>蘿絲（紅焰）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/ReneeSilver/base/ReneeSilver_1024.png" width="64"/><br/><sub>르네(백은)<br/>Renee (Argent)<br/>勒內（白銀）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Tasha/base/Tasha_1024.png" width="64"/><br/><sub>타샤<br/>Tasha<br/>塔莎</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Petra/base/Petra_1024.png" width="64"/><br/><sub>페트라<br/>Petra<br/>佩特拉</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Prim/base/Prim_1024.png" width="64"/><br/><sub>프림<br/>Prim<br/>弗里姆</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/SakuyoShin/base/SakuyoShin_1024.png" width="64"/><br/><sub>사쿠요(업화)<br/>Sakuyo (Inferno)<br/>櫻世（業火）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Sunny/base/Sunny_1024.png" width="64"/><br/><sub>순이<br/>Soonie<br/>順伊</sub></td>
</tr>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/Sharing/base/Sharing_1024.png" width="64"/><br/><sub>샤링<br/>Sharinne<br/>夏琳</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Amelia/base/Amelia_1024.png" width="64"/><br/><sub>비올레트<br/>Violette<br/>薇奧蕾特</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Yatogami/base/Yatogami_1024.png" width="64"/><br/><sub>야토가미 토카<br/>Tohka Yatogami<br/>夜刀神十香</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Talia/base/Talia_1024.png" width="64"/><br/><sub>탈리아<br/>Talia<br/>塔利亞</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Sigrid/base/Sigrid_1024.png" width="64"/><br/><sub>시그리드<br/>Sigrid<br/>希格莉德</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Seeha/base/Seeha_1024.png" width="64"/><br/><sub>시하<br/>Seeha<br/>西荷</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Ruri/base/Ruri_1024.png" width="64"/><br/><sub>루리<br/>Ruri<br/>魯莉</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Weiss/base/Weiss_1024.png" width="64"/><br/><sub>바이스<br/>Weiss<br/>拜斯</sub></td>
</tr>
<tr>
<td align="center"><img src="public/eversoul-assets/spirits/Velanna/base/Velanna_1024.png" width="64"/><br/><sub>벨라나<br/>Velanna<br/>貝拉納</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Vivienne/base/Vivienne_1024.png" width="64"/><br/><sub>비비안<br/>Vivienne<br/>薇薇安</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Xiaolian/base/Xiaolian_1024.png" width="64"/><br/><sub>소연<br/>Xiaolian<br/>小蓮</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Yuria/base/Yuria_1024.png" width="64"/><br/><sub>유리아<br/>Yuria<br/>尤里婭</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Sakuyo/base/Sakuyo_1024.png" width="64"/><br/><sub>사쿠요<br/>Sakuyo<br/>櫻世</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/YuriaApollyon/base/YuriaApollyon_1024.png" width="64"/><br/><sub>유리아(아폴리온)<br/>Yuria (Apollyon)<br/>尤里婭（阿巴頓）</sub></td>
<td align="center"><img src="public/eversoul-assets/spirits/Wheri/base/Wheri_1024.png" width="64"/><br/><sub>웨리<br/>Wheri<br/>威里</sub></td>
<td></td>
</tr>
</table>

정령마다 그림이 한 장으로 끝나지 않습니다. `base`(평소 모습), `costume`(의상), `raid`, `gacha`, `srg` 폴더로 나뉘어서 같은 정령이라도 여러 장의 그림이 준비되어 있습니다.

<p align="center">
  <img src="public/eversoul-assets/spirits/Adrianne/base/Adrianne_1024.png" width="110" alt="Adrianne base" />
  <img src="public/eversoul-assets/spirits/Adrianne/costume/Adrianne_Costume02_2048.png" width="110" alt="Adrianne costume" />
  <img src="public/eversoul-assets/spirits/Adrianne/gacha/Adrianne_Gacha_2048.png" width="110" alt="Adrianne gacha" />
  <img src="public/eversoul-assets/spirits/Adrianne/raid/Adrianne_Raid_2048.png" width="110" alt="Adrianne raid" />
</p>
<p align="center"><sub>아드리안 폴더에 있는 그림들 — 왼쪽부터 base, costume, gacha, raid</sub></p>

---

## 🚀 주요 기능

- 💻 **내 컴퓨터만으로 돌아가는 AI**: GPU가 없어도 `llama.cpp` 기반으로 대화를 만들어냅니다. CPU 물리 코어 수에 맞춰 스레드를 잡기 때문에 무리 없이 안정적으로 대답합니다.
- 🎭 **95명의 정령, 각자의 성격 그대로**: 이름과 등급, 종족, 직업은 물론 성우, 생일, 좋아하는 것까지 정령마다 정리해 둔 자료를 불러와서, 대화할 때마다 그 정령답게 말하도록 합니다.
- 🧠 **정령이 나와의 대화를 기억함**: 대화를 나눌 때마다 그 정령이 스스로 "기억할 만한 게 있었는지" 되짚어보고, 있으면 남겨둡니다. 기억이 쌓이면 한 번씩 다시 정리해서, 다음에 만났을 때도 그 기억을 그대로 안고 이야기합니다.
- 🌐 **언어를 바꿔도 그 정령 그대로**: 이름과 소개, 말투가 한국어·영어·중국어(번체/간체) 네 언어로 다 준비되어 있어서, 앱 언어를 바꾸면 정령의 이름과 소개도 바로 그 언어로 바뀝니다.
- 🧬 **대화가 쌓일수록 더 그 정령다워지는 학습**: Python 없이 순수 Rust(`candle`)로 Qwen2 모델을 직접 구현해서, 정령마다 나눈 대화를 바탕으로 내 컴퓨터에서 바로 미세조정(LoRA) 학습을 돌릴 수 있습니다.
- 📂 **대화는 전부 내 컴퓨터에 저장**: 나눈 대화, 정령 프로필, 말투 설정 모두 가벼운 SQLite 데이터베이스에 그대로 남습니다.
- 🖼️ **대화 배경도 그대로**: 에버소울 정식 일러스트 배경 522장을 언제든 꺼내서 대화창 분위기를 바꿀 수 있습니다.

<p align="center">
  <img src="public/eversoul-assets/backgrounds/talk/Talk_BG_Castle.png" width="150" alt="Talk BG Castle" />
  <img src="public/eversoul-assets/backgrounds/talk/Talk_BG_Library.png" width="150" alt="Talk BG Library" />
  <img src="public/eversoul-assets/backgrounds/talk/Talk_BG_Galaxy.png" width="150" alt="Talk BG Galaxy" />
  <img src="public/eversoul-assets/backgrounds/talk/Talk_BG_CherryBlossom.png" width="150" alt="Talk BG CherryBlossom" />
  <img src="public/eversoul-assets/backgrounds/talk/Talk_BG_Sanctum.png" width="150" alt="Talk BG Sanctum" />
  <img src="public/eversoul-assets/backgrounds/talk/Talk_BG_SkyArk.png" width="150" alt="Talk BG SkyArk" />
</p>

---

## 🏗 아키텍처

프론트엔드(React)와 백엔드(Rust)가 각각 동일한 이름의 도메인 모듈로 대칭 구성되어 있으며, Tauri IPC(`invoke`)로만 통신합니다.

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {'primaryColor': '#cde2fb', 'primaryBorderColor': '#2a78d6', 'primaryTextColor': '#0b0b0b', 'lineColor': '#52514e', 'clusterBkg': '#fcfcfb', 'clusterBorder': '#c3c2b7', 'fontFamily': 'system-ui, -apple-system, Segoe UI, sans-serif'}}}%%
flowchart TB
    subgraph FE["프론트엔드 · src/domains"]
        direction LR
        FE1["auth · chat · knowledge"]
        FE2["llm · persona · settings"]
        FE3["style · sync · training"]
        FE4["evertalk<br/>SpiritRoster · ChatStage · SettingsPanel"]
    end

    FE == "Tauri invoke<br/>31개 커맨드" ==> BE

    subgraph BE["백엔드 · src-tauri/src/domains + infrastructure"]
        direction LR
        BE1["auth · chat · knowledge"]
        BE2["llm · persona · settings"]
        BE3["style · sync · training"]
    end

    BE -- "대화방 · 메시지 · 정령 프로필 · 기억" --> DB[("SQLite<br/>eversoul.db")]
    BE -- "100% Prefix Reuse<br/>오프라인 데이터 영구 보존" --> CACHE[("KV Cache<br/>ai/cache/*.bin")]
    BE -- "로컬 컨텍스트 조립 추론" --> LLM["GGUF 로컬 모델<br/>Qwen2.5-3B-Korean<br/>llama.cpp"]

    classDef feStyle fill:#cde2fb,stroke:#2a78d6,stroke-width:2px,color:#0b0b0b
    classDef beStyle fill:#e3ddf7,stroke:#4a3aa7,stroke-width:2px,color:#0b0b0b
    classDef dbStyle fill:#c9f0d8,stroke:#008300,stroke-width:2px,color:#0b0b0b
    classDef llmStyle fill:#fbdcc9,stroke:#eb6834,stroke-width:2px,color:#0b0b0b
    classDef cacheStyle fill:#fff4cc,stroke:#ffb703,stroke-width:2px,color:#0b0b0b

    class FE1,FE2,FE3,FE4 feStyle
    class BE1,BE2,BE3 beStyle
    class DB dbStyle
    class LLM llmStyle
    class CACHE cacheStyle
```

- **로컬 DB 경로**: OS별 앱 데이터 디렉터리 하위 `database/eversoul.db` (디버그 빌드 시 매 실행마다 초기화).
- **설정 파일**: 앱 데이터 디렉터리 하위 `config/settings.ini` (`rust-ini`로 읽기/쓰기, 기본 정령·활성 스타일·언어 저장).
- **KV Cache 저장소**: 앱 실행 디렉터리 하위 `ai/cache/` (정령별 프롬프트 조립 결과를 `.bin` 물리 파일로 영구 보존하여 Prefix Token 부분 재사용 및 연산량 최소화 100% 달성).
- **비동기 런타임 구조**: 백엔드의 LLM 연산은 `tauri::async_runtime::spawn_blocking` 워커로 분리되어 메인 UI 스레드 논블로킹 보장.

정령 데이터 빌드 파이프라인, 대화 처리 시퀀스, LoRA 학습 흐름, 데이터베이스 구조까지 더 자세한 다이어그램은 [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)에 있습니다.

---

## 🛠 기술 스택

### Frontend Stack

- **Framework**: `React 19.1` + `TypeScript 6.0` + `Vite 8`
- **State Management**: `TanStack React Query v5`(비동기 서버 상태), `Zustand v5`(전역 클라이언트 상태)
- **Styling**: `Tailwind CSS v4`(`@tailwindcss/vite`) + `clsx`(클래스 믹싱)
- **Icons**: `lucide-react`
- **Tauri Plugins**: `@tauri-apps/plugin-dialog`, `plugin-fs`, `plugin-opener`, `plugin-shell`

### Desktop Runtime & Backend Stack

- **Core Runtime**: `Tauri v2` (Rust 2021 edition), release 빌드는 `codegen-units=1` + `lto=true` + `opt-level=3` + `panic=abort` + `strip`로 최적화.
- **Local Database**: `SQLite3` (`rusqlite` bundled)
- **HTTP Client**: `reqwest`(rustls, json, stream 기능)
- **AI Inference Engine**: `llama.cpp` (`llama-cpp-2` C-bindings, GGUF 포맷) + `num_cpus`(물리 코어 기준 스레드 산정)
- **On-device Fine-tuning**: `candle-core` / `candle-nn` 0.8 (Qwen2 아키텍처 + LoRA 어댑터 직접 구현), `hf-hub`, `tokenizers`(BPE, `onig` 기능)
- **Serialization / Utilities**: `serde`, `serde_json`, `anyhow`, `thiserror`, `tracing` + `tracing-subscriber`, `uuid`, `directories`, `sha2`, `hex`, `flate2`, `rust-ini`

---

## 📦 로컬 모델

고품질 한국어 성능을 위해 단일 고정 모델을 씁니다. 배포판에는 이 모델이 이미 포함되어 있어 따로 받을 필요가 없습니다.

- **이름**: `MyeongHo0621/Qwen2.5-3B-Korean Q4_K_M`
- **위치**: `ai/model/qwen25-3b-korean-Q4_K_M.gguf`

---

## 💻 실행 및 빌드 가이드

### 빌드 사전 요구사항

로컬 LLM 추론 바인딩(`llama-cpp-2`)을 빌드하기 위해 아래 도구들의 사전 설치가 필요합니다.

- [Visual Studio Build Tools](https://visualstudio.microsoft.com/downloads/) (C++ 컴파일러 포함)
- [CMake](https://cmake.org/download/) (버전 3.20 이상)
- [Clang](https://releases.llvm.org/download.html) (Bindgen용 C/C++ 파서)

### 의존성 설치

```bash
npm install
```

### 개발 모드 실행

```bash
npm run tauri dev
```

### 상용 배포 빌드

```bash
npm run tauri build
```

---

## 🧩 정령(페르소나) 데이터 스키마

정령은 종족(`race`)에 따라 일곱 갈래로 나뉩니다.

<table>
<tr>
<td align="center"><img src="public/eversoul-assets/ui/race-badges/beast.svg" width="64" alt="야수형" /><br/><sub>야수형</sub></td>
<td align="center"><img src="public/eversoul-assets/ui/race-badges/human.svg" width="64" alt="인간형" /><br/><sub>인간형</sub></td>
<td align="center"><img src="public/eversoul-assets/ui/race-badges/elf.svg" width="64" alt="요정형" /><br/><sub>요정형</sub></td>
<td align="center"><img src="public/eversoul-assets/ui/race-badges/undead.svg" width="64" alt="불사형" /><br/><sub>불사형</sub></td>
<td align="center"><img src="public/eversoul-assets/ui/race-badges/chaos.svg" width="64" alt="혼돈형" /><br/><sub>혼돈형</sub></td>
<td align="center"><img src="public/eversoul-assets/ui/race-badges/angel.svg" width="64" alt="천사형" /><br/><sub>천사형</sub></td>
<td align="center"><img src="public/eversoul-assets/ui/race-badges/demon.svg" width="64" alt="악마형" /><br/><sub>악마형</sub></td>
</tr>
</table>

앱이 실제로 대화할 때 정령의 이름·성격·말투를 읽어오는 곳은 SQLite `persona_profile.raw_json` 컬럼입니다. 정령 목록을 조회할 때마다 `PersonaService::get_available_personas`가 `personas.bin`에서 다시 불러와 이 컬럼에 매번 덮어써서(upsert) 저장하고, 실제 LLM 시스템 프롬프트는 `PersonaService::build_localized_system_prompt`가 이 `raw_json`을 파싱해서 조립합니다.

이 데이터의 원본은 `data/personas/*.json` 95개 파일이며, `tools/build_complete_personas.cjs`가 이를 `ko / en / zh_tw / zh_cn` 4개 언어 배열(`LANGUAGES`)로 정규화해 `personas.bin`으로 빌드해 둡니다. 아래는 그 원본 JSON 하나(아드리안)의 실제 필드 구조입니다.

```json
{
  "id": "5020",
  "name": "아드리안",
  "name_en": "Adrianne",
  "grade": "에픽",
  "race": "천사형",
  "class": "디펜더",
  "sub_class": "광역",
  "stat": "힘",
  "profile": {
    "nick_name": "정의의 빛",
    "constellation": "천칭자리",
    "union": "에델 가드",
    "birthday": "1017",
    "height": 167,
    "weight": 51,
    "cv_ko": "이명호",
    "cv_jp": "Eri Kitamura",
    "like": ["강아지", "감동 실화"],
    "dislike": ["범죄", "악인"],
    "hobby": ["영지 순찰"],
    "speciality": ["멋진 포즈 연구"]
  },
  "personality": { "description": "...", "greeting": "..." },
  "speech_patterns": ["...", "..."],
  "i18n": {
    "name": {
      "ko": "아드리안",
      "en": "Adrianne",
      "zh_tw": "阿德里安",
      "zh_cn": "阿德里安"
    },
    "grade": { "ko": "에픽", "en": "Epic", "zh_tw": "史詩", "zh_cn": "史詩" },
    "race": {
      "ko": "천사형",
      "en": "Angel",
      "zh_tw": "天使型",
      "zh_cn": "天使型"
    },
    "class": {
      "ko": "디펜더",
      "en": "Defender",
      "zh_tw": "捍衛者",
      "zh_cn": "捍衛者"
    },
    "profile": {
      "nick_name": {
        "ko": "정의의 빛",
        "en": "Light of Justice",
        "zh_tw": "正義之光",
        "zh_cn": "正義之光"
      },
      "constellation": {
        "ko": "천칭자리",
        "en": "Libra",
        "zh_tw": "天秤座",
        "zh_cn": "天秤座"
      }
    }
  }
}
```

- `i18n` 블록은 필드 이름을 키로 두고 그 아래 `{ ko, en, zh_tw, zh_cn }` 4개 언어 값을 나란히 갖는 **필드-우선 구조**이며, `name` · `grade` · `race` · `class` · `sub_class` · `stat`은 물론 `profile.nick_name` · `profile.constellation` · `profile.union` · `profile.cv_ko` · `profile.cv_jp` · `profile.like` · `profile.dislike` · `profile.hobby` · `profile.speciality`까지 세부 필드 단위로 번역이 존재합니다.
- 화면에 보여줄 때는 `src/domains/persona/logic.ts`의 `parseSpiritDetail`이 이 `raw_json`을 파싱해 언어별로 골라내고, 실제 LLM에게 보낼 시스템 프롬프트는 이것과 별개로 Rust 백엔드의 `PersonaService::build_localized_system_prompt`가 `raw_json`을 다시 파싱해 직접 조립합니다 — 두 곳 다 최종적으로는 SQLite의 `raw_json`을 소스로 씁니다.
- 정령별 원화는 `public/eversoul-assets/spirits/{영문명}/` 하위에 `base`(기본 일러스트 512/1024/2048), `costume`(코스튬), `gacha`(가챠 연출), `raid`(레이드 연출), `srg`(스토리) 등 카테고리 폴더로 분리되어 있으며, `LoadableAssetImage` 컴포넌트(`src/domains/evertalk/components/LoadableAssetImage.tsx`)가 후보 경로 배열을 순차 시도(`useFirstLoadableImage`)해 존재하는 첫 이미지를 렌더링합니다.

---

## 📌 버전 관리 규칙

이 저장소는 **커밋 1회당 patch 버전 +1**을 원칙으로 합니다. `package.json` · `src-tauri/Cargo.toml` · `src-tauri/tauri.conf.json` 세 파일의 `version` 필드는 항상 동일한 값으로 동기화되어야 하며, 기능 변경이 포함된 커밋을 생성할 때마다 세 파일을 함께 갱신합니다.

| 버전  | 커밋                                      |
| ----- | ----------------------------------------- |
| 0.0.1 | `first`                                   |
| 0.0.2 | `초기세팅`                                |
| 0.0.3 | `초기세팅2`                               |
| 0.0.4 | `초기세팅3`                               |
| 0.0.5 | `초기셋팅4`                               |
| 0.0.6 | `update_i18n : en , kr , zh_tw , zh_cn`   |
| 0.0.7 | 다국어 README 3종 정비 + 버전 규칙 문서화 |
| 0.0.7 | `up` |
| 0.0.8 | `bugfix` |
| 0.0.9 | `up` |
| 0.0.10 | `fix` |
| 0.0.11 | `1` |
| 0.0.12 | `초기릴리즈` |
| 0.0.13 | `클린` |
| 0.0.14 | `feat:` |

---

## 📄 라이선스

This project is licensed under the **Apache License 2.0**.
GGUF Model (`Qwen2.5-3B-Korean`) is created by `MyeongHo0621` and distributed under **Apache License 2.0**.
