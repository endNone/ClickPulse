'use strict';

// Each translation describes a visible choice a designer can execute. Directions
// stay separate from the user's dates, counts and the geometry of the data.
const ClickPulseReportRecipes = (() => {
  const locales = ['en', 'zh-CN', 'zh-TW', 'ja', 'ko', 'fr', 'de', 'es'];

  function text(variants) {
    return Object.freeze(Object.fromEntries(locales.map((locale, i) => [locale, variants[i]])));
  }

  // Localized lines pair the compact picker label with the longer prompt directive.
  function words(...variants) {
    return Object.freeze(Object.fromEntries(locales.map((locale, i) => {
      const [label, instruction] = variants[i].split(' | ');
      return [locale, Object.freeze({label, instruction})];
    })));
  }

  const mediumWords = {
    risograph: words(
      'Risograph | Build the poster from flat spot inks, broad shapes and fine halftone dots. Offset ink layers only on decorative shapes; keep lettering and data marks sharp.',
      '孔版叠印 | 用专色块和细网点表现孔版印刷。装饰边缘可轻微错版，文字和图表刻度要清晰对齐。',
      '孔版套印 | 用特別色塊與細網點呈現孔版印刷。裝飾邊緣可略微錯版，文字與圖表刻度要清楚對齊。',
      'リソグラフ | 特色の面と細かな網点で刷りの質感を出す。版ずれは装飾だけに使い、文字と目盛りを鮮明にする。',
      '리소그래프 | 별색 면과 작은 망점으로 인쇄 질감을 만든다. 장식의 잉크만 살짝 어긋나게 하고 글자와 눈금은 정확히 맞춘다.',
      'Risographie | Composez en aplats et en trames fines. Réservez le léger décalage des encres aux ornements et gardez lettres et repères nets.',
      'Risodruck | Arbeite mit Volltonflächen und feinen Rasterpunkten. Lasse nur dekorative Farblagen leicht versetzt drucken und halte Beschriftungen und Skalen scharf.',
      'Risografía | Combina tintas planas y tramas de puntos finos. Desplaza levemente solo las capas decorativas y conserva nítidos los textos y las escalas.'),
    letterpress: words(
      'Letterpress | Use a broadside of thick uncoated paper, stout serif numerals and thin printer’s rules. Suggest a shallow impression at the edges of large type.',
      '凸版印刷 | 把版面做成厚纸单页，用粗壮的衬线数字和细分隔线。大字边缘带轻微压痕，保留平整的图表区。',
      '凸版印刷 | 採用厚紙單頁、厚實的襯線數字與細分隔線。大字邊緣略帶壓印感，圖表區保持平整。',
      '活版印刷 | 厚手の紙に太いセリフ数字と細い罫線を組む。大きな文字の縁に浅い圧痕を表現する。',
      '활판 인쇄 | 두꺼운 종이 한 장에 굵은 세리프 숫자와 가는 괘선을 조합한다. 큰 글자 가장자리에 얕은 눌림을 표현한다.',
      'Impression typographique | Évoquez une feuille épaisse non couchée avec des chiffres à empattements massifs et de fins filets. Suggérez une légère empreinte autour des grands caractères.',
      'Buchdruck | Setze kräftige Serifenziffern und feine Linien auf dickes Naturpapier. Deute an den Rändern großer Lettern einen leichten Prägedruck an.',
      'Impresión tipográfica | Usa papel grueso sin estucar, cifras robustas con remates y filetes finos. Sugiere un leve hundimiento en los bordes de los tipos grandes.'),
    transit: words(
      'Transit wayfinding | Borrow the visual grammar of a station sign: a strong route band, compact destination-style labels and generously spaced numerals. Use only the supplied dates or time bins as route markers.',
      '公共交通导视 | 借用站牌的色带、紧凑标签和疏朗数字。沿路线只标给出的日期或时段。',
      '交通導引 | 運用站牌的色帶、簡潔標籤與寬鬆排列的數字。路線上只標示提供的日期或時段。',
      '交通サイン | 駅の案内板のように太い路線帯、簡潔なラベル、ゆったりした数字を使う。路線上の目印には与えられた日付や時間帯を使う。',
      '교통 안내판 | 역 안내판처럼 굵은 노선 띠와 짧은 표기, 간격이 넉넉한 숫자를 쓴다. 노선 표식에는 제공된 날짜나 시간대만 넣는다.',
      'Signalétique de transport | Empruntez aux panneaux de gare leur bande de ligne, leurs libellés compacts et leurs chiffres espacés. Les dates ou créneaux fournis servent de seuls jalons.',
      'Verkehrsleitsystem | Nutze Linienbänder, kompakte Zielschilder und großzügig gesetzte Ziffern wie auf einem Bahnhofsschild. Als Stationen dienen nur die gelieferten Daten oder Zeitfenster.',
      'Señalética de transporte | Recurre a una banda de línea, rótulos breves y cifras espaciadas como en una estación. Usa únicamente las fechas o franjas dadas como paradas.'),
    swiss: words(
      'Swiss exhibition poster | Align oversized sans-serif figures to a strict asymmetric grid. Use a bold rectangle, flush-left labels and deliberate empty space.',
      '瑞士展览海报 | 将大号无衬线数字压在不对称网格上。用醒目的矩形、左对齐的标签和充足留白组织信息。',
      '瑞士展覽海報 | 讓大型無襯線數字對齊不對稱格線。以鮮明矩形、靠左標籤與充足留白安排資訊。',
      'スイス風展覧会ポスター | 大きなサンセリフ数字を非対称グリッドに合わせる。太い矩形、左揃えのラベルと余白で構成する。',
      '스위스 전시 포스터 | 큰 산세리프 숫자를 비대칭 그리드에 맞춘다. 선명한 직사각형과 왼쪽 정렬 표기, 넉넉한 여백으로 구성한다.',
      'Affiche suisse | Alignez de grands chiffres sans empattement sur une grille asymétrique. Un rectangle franc, des légendes ferrées à gauche et du blanc rythment la page.',
      'Schweizer Ausstellungsplakat | Richte große serifenlose Ziffern an einem asymmetrischen Raster aus. Gliedere die Fläche mit einem kräftigen Rechteck, linksbündigem Text und viel Freiraum.',
      'Cartel suizo | Alinea grandes cifras sin remates en una retícula asimétrica. Compón con un rectángulo contundente, rótulos a la izquierda y espacio libre.'),
    blueprint: words(
      'Drafting sheet | Draw fine ruled construction lines and squared-off labels around the chart. Keep the construction grid faint; put measured data on a heavier stroke.',
      '工程制图 | 用细辅助线和方框标签围出绘图区，让底网隐退，用较粗线条绘制实测数据。',
      '工程製圖 | 用細輔助線與方框標籤圍出繪圖區。淡化底層格線，以較粗線條表現實測資料。',
      '製図用紙 | 細い補助線と角形のラベルで作図範囲を示す。背景の方眼は淡くし、実測値の線を太くする。',
      '설계 도면 | 얇은 보조선과 각진 표기로 차트 영역을 잡는다. 배경 격자는 흐리게, 실제 데이터 선은 굵게 그린다.',
      'Plan de dessin industriel | Encadrez la zone tracée de traits de construction et de cartouches anguleux. Gardez le quadrillage léger et renforcez le trait des données mesurées.',
      'Technische Zeichnung | Rahme die Grafik mit feinen Hilfslinien und eckigen Beschriftungsfeldern. Das Grundraster bleibt blass, Messwerte erhalten kräftigere Striche.',
      'Plano técnico | Delimita el gráfico con trazos auxiliares y rótulos rectangulares. Atenúa la cuadrícula y da más grosor al trazo de los datos medidos.'),
    woodcut: words(
      'Woodcut | Carve bold silhouettes and directional hatching into the margins. Reserve a clear unhatched area for the exact figures and chart.',
      '木刻版画 | 用刀刻般的轮廓和定向排线丰富页边。数字与图表所在的区域保留干净平面。',
      '木刻版畫 | 以刀刻般的輪廓與定向排線刻畫頁邊。數字和圖表所在區域保留乾淨平面。',
      '木版画 | 大胆な輪郭と彫り跡のような斜線を余白に置く。数値とグラフの部分は線を敷かずに空ける。',
      '목판화 | 가장자리에 굵은 윤곽과 한 방향으로 파낸 빗금을 넣는다. 수치와 차트가 놓이는 면은 깨끗이 비워 둔다.',
      'Gravure sur bois | Taillez les marges en silhouettes franches et en hachures directionnelles. Réservez une surface sans hachures aux chiffres et au graphique.',
      'Holzschnitt | Gestalte die Ränder mit kräftigen Schnittformen und gerichteten Schraffuren. Zahlen und Grafik stehen auf einer ruhigen, ungeschrafften Fläche.',
      'Xilografía | Talla siluetas contundentes y rayados direccionales en los márgenes. Reserva una zona lisa para los valores exactos y el gráfico.'),
    eink: words(
      'E-ink reader | Build a matte e-paper display in a slim molded bezel. Use crisp pixel edges on the oversized total, solid text and a flat plot with discreet dotted rules.',
      '电子墨水屏 | 用薄模塑边框围住哑光电子纸。总数数字边缘略带像素感，正文实色清晰，图表平面上只用浅点线分隔。',
      '電子墨水屏 | 以薄型成形邊框包住霧面電子紙。總數字緣帶些像素感，正文清晰實印，平面的圖區以淡點線分隔。',
      '電子ペーパー | 薄い成形ベゼルにマットな電子ペーパーを収める。大きな合計数字はピクセルの輪郭を生かし、本文は明瞭に、グラフの罫線は淡い点線にする。',
      '전자잉크 화면 | 얇은 테두리 안에 무광 전자종이 화면을 둔다. 큰 합계 숫자는 픽셀 윤곽을 살리고 본문은 또렷하게, 평면 도표의 구분선은 옅은 점선으로 그린다.',
      'Liseuse à encre électronique | Encadrez le papier électronique mat d’un mince boîtier moulé. Dessinez le grand total à bords pixellisés, le texte en aplat et le graphique à plat sur de fins pointillés.',
      'E-Paper-Display | Fasse das matte E-Paper in einen schmalen Gehäuserand. Gib der großen Summe klare Pixelkanten und nutze deckende Schrift und feine Punktlinien im flachen Diagramm.',
      'Pantalla de tinta electrónica | Enmarca el papel electrónico mate con un bisel fino. Da al total grande bordes pixelados y usa texto sólido y líneas punteadas discretas en el gráfico plano.'),
    textile: words(
      'Woven textile | Build the borders from warp-and-weft threads and use a stitched edge. Keep the data on a smooth woven label with crisp printed numerals.',
      '织物标签 | 以经纬线和缝边织出边框。把数据排在平整的布标上，数字采用清晰的印字。',
      '織物標籤 | 以經緯線與縫邊織出邊框。把資料排在平整的布標上，數字使用清晰印字。',
      '織物ラベル | 縦糸と横糸、縫い目で縁を構成する。データは平らな布ラベルに置き、数字は鮮明な活字にする。',
      '직물 라벨 | 날실과 씨실, 박음질로 테두리를 짠다. 데이터는 매끈한 천 라벨에 놓고 선명한 활자로 숫자를 찍는다.',
      'Étiquette tissée | Composez une bordure en fils de chaîne et de trame, terminée par une couture. Imprimez les données en chiffres nets sur une étiquette lisse.',
      'Webetikett | Baue den Rand aus Kett- und Schussfäden mit einer sichtbaren Naht. Drucke die Daten als klare Ziffern auf ein glattes Webetikett.',
      'Etiqueta tejida | Teje el borde con urdimbre y trama y remátalo con una costura. Imprime cifras nítidas sobre una etiqueta de tejido liso.'),
    glass: words(
      'Frosted glass dashboard | Layer sleek dark-tinted frosted glass bento cards over a deep atmospheric gradient backdrop. Use crisp hairline borders, subtle ambient glow, solid high-contrast figures, and tactile 3D keycap badges—no white outer border or muddy white drop-shadows.',
      '暗夜磨砂玻璃 | 在深邃氛围渐变底色上叠放精致的深色磨砂玻璃便当盒卡片，搭配利落细描边、柔和环境微光、高对比实色数字与立体键帽徽章，不加白色外框或发灰的白色背景阴影。',
      '暗夜霧面玻璃 | 在深邃氛圍漸層底色上疊放精緻的深色霧面玻璃便當盒卡片，搭配俐落細描邊、柔和環境微光、高對比實色數字與立體鍵帽徽章，不加白色外框或發灰的白色背景陰影。',
      'すりガラスのパネル | 深みのあるグラデーション背景に洗練されたダークすりガラスのベントカードを配置。繊細な枠線とほのかな光、高コントラストの数字、立体キーキャップを使い、白い外枠や白ぼけ影は入れない。',
      '다크 글래스 패널 | 깊은 분위기 그라데이션 배경 위에 세련된 다크 반투명 유리 벤토 카드를 배치한다. 얇은 테두리와 은은한 광채, 고대비 숫자와 입체 키캡을 쓰고 흰 외곽선이나 뿌연 흰 그림자는 넣지 않는다.',
      'Panneau de verre dépoli | Superposez des cartes bento en verre dépoli sombre sur un fond atmosphérique profond, avec bordures fines, lueur douce, chiffres nets et touches 3D, sans cadre blanc ni ombre blanche diffuse.',
      'Milchglas-Dashboard | Lege dunkle Milchglas-Bento-Karten über einen tiefen atmosphärischen Verlauf mit feinen Linien, sanftem Schimmer, klaren Ziffern und 3D-Tastenkappen — ohne weißen Rand oder weiße Schattenkästen.',
      'Panel de vidrio esmerilado | Superpón tarjetas bento de vidrio esmerilado oscuro sobre un fondo atmosférico profundo con bordes finos, brillo suave, cifras nítidas y teclas 3D, sin marco blanco ni sombras blancas difusas.'),
    contact: words(
      'Photo contact sheet | Frame the supplied measurements like adjacent exposures on a contact sheet. Use narrow gutters, film-edge texture and plain captions under each frame.',
      '摄影接触印样 | 把给出的数据分组，像相邻底片般排版。用窄间距、胶片边纹和简短图注组织画面。',
      '攝影接觸印樣 | 將提供的資料分組，如相鄰底片般排列。用窄間距、膠片邊紋與簡短圖說安排畫面。',
      'コンタクトシート | 与えられた測定値を、並んだ写真のコマのように区切る。細い余白、フィルム縁の質感、各コマの短いキャプションを使う。',
      '사진 밀착 인화지 | 제공된 수치를 인화지의 나란한 프레임처럼 묶는다. 좁은 간격과 필름 가장자리 질감, 프레임 아래 간단한 설명을 사용한다.',
      'Planche-contact | Cadrez les mesures fournies comme des vues voisines sur une planche-contact. Utilisez des gouttières étroites, un bord de pellicule et des légendes sobres.',
      'Fotokontaktbogen | Rahme die Messwerte wie benachbarte Aufnahmen auf einem Kontaktbogen. Nutze schmale Stege, Filmrandstruktur und knappe Bildunterschriften.',
      'Hoja de contactos | Enmarca las mediciones como exposiciones contiguas en una hoja de contactos. Utiliza calles estrechas, textura de borde de película y pies sencillos.'),
    papercut: words(
      'Cut-paper collage | Layer two or three large cut-paper shapes with discreet edge shadows. Put labels and charts on the top flat layer.',
      '剪纸拼贴 | 叠放两三块大幅剪纸，用轻微边缘阴影体现层次。文字和图表统一放在最上层平面。',
      '剪紙拼貼 | 疊放兩三塊大幅剪紙，透過細微邊緣陰影呈現層次。文字和圖表放在最上層平面。',
      '切り紙コラージュ | 大きな切り紙を二、三枚重ね、縁の浅い影で奥行きを出す。ラベルとグラフは平らな最上層に載せる。',
      '종이 오리기 콜라주 | 큰 색종이 두세 장을 겹치고 가장자리에 약한 그림자를 준다. 글자와 차트는 맨 위의 평평한 면에 놓는다.',
      'Collage de papiers découpés | Superposez deux ou trois grandes formes découpées avec de légères ombres de bord. Les légendes et le graphique occupent le plan supérieur.',
      'Scherenschnitt-Collage | Schichte zwei oder drei große Papierformen mit feinen Kantenschatten. Setze Zahlen, Grafik und Beschriftung auf die flache oberste Lage.',
      'Collage de papel recortado | Superpón dos o tres formas grandes de papel con sombras suaves en los bordes. Coloca el gráfico y los rótulos sobre la capa superior lisa.'),
    lcd: words(
      'Pocket LCD | Set a segmented total inside a recessed calculator screen with a narrow chamfered rim. Use small plain labels and sharply drawn marks as if etched on the display.',
      '口袋液晶屏 | 像袖珍计算器一样，把分段式大数字放在内嵌液晶屏中，屏边用窄倒角。短标签和图形标记像蚀刻一样清晰。',
      '口袋液晶屏 | 如同袖珍計算機，以分段式大數字顯示總數，液晶屏嵌在窄斜邊內。短標籤與圖形刻線清晰俐落。',
      'ポケット液晶 | 細い面取り縁を備えた電卓風の液晶に、セグメント数字で合計を表示する。小さなラベルと刻印風の図形をはっきり描く。',
      '포켓 LCD | 모서리를 좁게 깎은 오목한 계산기 화면에 합계를 세그먼트 숫자로 표시한다. 짧은 표기와 도표 표식은 화면에 새긴 듯 또렷하게 그린다.',
      'Écran LCD de poche | Inscrivez le total en grands segments dans un écran de calculatrice encastré à bord biseauté. Gravez visuellement des libellés simples et des repères nets sur l’écran.',
      'Taschen-LCD | Zeige die Summe in Segmentziffern auf einem eingelassenen Rechnerdisplay mit schmaler Fase. Setze kurze schlichte Labels und scharfe Diagrammmarken wie geätzte Linien auf das Display.',
      'LCD de bolsillo | Presenta el total en dígitos segmentados dentro de una pantalla de calculadora con bisel fino. Traza rótulos breves y marcas precisas como grabadas en la pantalla.'),
    keycaps: words(
      'Isometric keycaps | Arrange a few sculpted keycaps at one isometric angle along the outer edge, with matte faces and short shadows. Keep values, chart and labels on a flat front-facing plane.',
      '等距立体键帽 | 在页边沿同一等距视角排几颗立体键帽，表现哑光键面和短投影。数值、图形和标签统一放在面向读者的平面上。',
      '等角立體鍵帽 | 在邊緣以同一等角視角排幾顆立體鍵帽，呈現霧面與短陰影。數值、圖表和標籤統一放在正向讀者的平面。',
      'アイソメトリックキーキャップ | 外縁に数個の立体キーを同じ等角投影で並べ、マットな面と短い影を描く。数値、グラフ、ラベルは正面を向く平面に置く。',
      '등각 키캡 | 외곽에 무광 키캡 몇 개를 같은 등각 투영으로 배치하고 짧은 그림자를 준다. 수치와 도표, 표기는 정면을 향한 동일 평면에 놓는다.',
      'Touches en isométrie | Disposez quelques touches sculptées en marge sous un même angle isométrique, avec des faces mates et des ombres courtes. Placez chiffres, graphique et légendes sur un plan frontal.',
      'Isometrische Tastenkappen | Ordne einige modellierte Tasten am Rand in derselben isometrischen Ansicht an, mit matten Oberflächen und kurzen Schatten. Zahlen, Grafik und Beschriftung stehen frontal auf einer Ebene.',
      'Teclas isométricas | Coloca unas pocas teclas en el margen bajo el mismo ángulo isométrico, con caras mates y sombras cortas. Traza cifras, gráfico y rótulos sobre un plano frontal.'),
    terminal: words(
      'CRT terminal | Place monospaced figures on a phosphor screen with a softly rounded bezel. Keep the glow and scan lines faint at the edges. Align numeric columns by their final digit.',
      'CRT 终端 | 在圆角显像管上排等宽数字，边缘带淡磷光和扫描线。数字列按个位对齐，绘图区保持清晰。',
      'CRT 終端機 | 在圓角映像管上排等寬數字，邊緣帶淡磷光與掃描線。數字欄按個位對齊，圖區清楚。',
      'ブラウン管端末 | 丸みのある蛍光面に等幅数字を表示する。縁の発光と走査線は薄く、数値列の末尾を揃える。',
      'CRT 터미널 | 둥근 브라운관 안에 고정폭 숫자를 놓는다. 가장자리의 인광과 주사선은 약하게 살리고 숫자 열의 일의 자리를 맞춘다.',
      'Terminal cathodique | Placez des chiffres à chasse fixe sur un écran au phosphore à bord arrondi. Limitez la lueur et les lignes de balayage aux bords. Alignez les unités de chaque colonne.',
      'Röhrenterminal | Setze Monospace-Ziffern auf einen Phosphorschirm mit gerundetem Rand. Halte Leuchten und Abtastlinien am Rand dezent. Richte Zahlenspalten nach der letzten Stelle aus.',
      'Terminal de tubo | Sitúa las cifras monoespaciadas en una pantalla de fósforo con bordes redondeados. Usa un brillo y líneas de barrido tenues en los márgenes y alinea las unidades por columna.'),
    newsprint: words(
      'Newspaper graphics | Build a newspaper page from a strong masthead rule, narrow columns and a compact chart caption. Keep the numbers clear against a lightly grained ground.',
      '报刊信息图 | 用报头横线、窄栏和短图注组织版面。在带少量纸纹的底色上清楚排印数字。',
      '報刊資訊圖 | 用報頭橫線、窄欄與短圖說安排版面。在略帶紙紋的底色上清楚排印數字。',
      '新聞インフォグラフィック | 題字下の太い罫、細い段組み、短い図注で紙面を組む。かすかな紙目の上でも数字は鮮明にする。',
      '신문 인포그래픽 | 제호 아래 굵은 선, 좁은 단과 간단한 캡션으로 지면을 구성한다. 가벼운 종이 질감 위에 숫자는 또렷하게 인쇄한다.',
      'Infographie de presse | Structurez la page avec un filet de manchette, des colonnes étroites et une courte légende. Les chiffres restent nets sur le grain discret du papier.',
      'Zeitungsinfografik | Baue die Seite mit einer kräftigen Kopfzeile, schmalen Spalten und einer kurzen Bildlegende auf. Halte Zahlen auf dezent gekörntem Papier klar lesbar.',
      'Infografía de prensa | Organiza la página con un filete de cabecera, columnas estrechas y un pie de gráfico breve. Mantén las cifras claras sobre un leve grano de papel.'),
    album: words(
      'Record sleeve | Treat the total as the album title. Crop a grooved record circle into the outer edge and letter the supplied time labels in the compact type of a sleeve spine.',
      '唱片封套 | 把总数当作专辑名排版。一角露出带细纹的唱片圆弧，已有时段标签用唱片封脊风格的窄体小字。',
      '唱片封套 | 將總數當作專輯名排版。一角露出帶細紋的唱片圓弧，已有時段標籤採用唱片封套書脊般的窄體字。',
      'レコードジャケット | 合計をアルバム名のように組む。端に溝のあるレコードの円弧をのぞかせ、与えられた時間ラベルはジャケットの背を思わせる細身の書体にする。',
      '레코드 재킷 | 합계를 앨범명처럼 배치한다. 가장자리에 홈이 있는 음반의 일부를 드러내고 주어진 시간 표기는 재킷 옆면처럼 폭이 좁은 글씨로 적는다.',
      'Pochette de disque | Composez le total comme un titre d’album. Laissez dépasser un arc de vinyle strié au bord et inscrivez les repères temporels fournis en caractères étroits comme sur la tranche d’une pochette.',
      'Plattenhülle | Setze die Summe wie einen Albumtitel. Lasse am Rand einen gerillten Plattenbogen ins Bild ragen und setze die vorgegebenen Zeitangaben in schmaler Schrift wie auf einem Hüllenrücken.',
      'Funda de disco | Trata el total como el título del álbum. Deja asomar un arco de vinilo estriado en el borde y rotula los tiempos dados con la letra estrecha del lomo de una funda.'),
    inkwash: words(
      'Chinese ink-wash painting | Paint misty mountain-and-water sumi-e ink washes on warm xuan rice paper with a vermilion seal stamp accent. Keep the chart and figures crisp in deep sumi ink over clean paper—no white glow boxes.',
      '中国水墨画 | 以温润宣纸为底，融入淡墨山水晕染与朱砂闲章点缀。核心数字与图表用沉稳浓墨清晰绘制在干净留白处，不加白色阴影框。',
      '中國水墨畫 | 以溫潤宣紙為底，融入淡墨山水暈染與硃砂閒章點綴。核心數字與圖表用沉穩濃墨清晰繪製於留白處，不加白色陰影框。',
      '水墨画 | 温かみのある宣紙に淡墨の山水のにじみと朱印を添える。数字とグラフは余白に濃墨で鮮明に描き、白い影枠は使わない。',
      '수묵화 | 따뜻한 한지 위에 옅은 먹의 산수 번짐과 붉은 낙관 포인트를 더한다. 숫자와 차트는 여백 위에 진한 먹색으로 선명하게 그린다.',
      'Lavis chinois | Composez des lavis d’encre brumeux sur papier de riz chaud avec un sceau vermillon. Gardez chiffres et graphique nets à l’encre noire sans cadres blancs.',
      'Chinesische Tuschemalerei | Gestalte zarte Tuscheverläufe auf warmem Reispapier mit zinnoberrotem Stempelakzent. Halte Zahlen und Diagramm scharf in tiefer Tusche ohne weiße Schattenkästen.',
      'Pintura china a la tinta | Crea aguadas de tinta neblinosa sobre papel de arroz cálido con un sello bermellón. Mantén cifras y gráfico nítidos en tinta oscura sin recuadros blancos.'),
    ukiyoe: words(
      'Ukiyo-e woodblock print | Style the poster as a classic Japanese ukiyo-e woodblock print with Prussian blue waves, stylized clouds, bold sumi outlines and warm washi grain. Frame the data cleanly in flat woodblock panels.',
      '浮世绘木刻 | 以传统浮世绘木版画质感呈现：普鲁士蓝海浪、祥云纹样、利落墨线轮廓与和纸肌理，用平整套印色块清晰框出数据与图表。',
      '浮世繪木刻 | 以傳統浮世繪木版畫質感呈現：普魯士藍海浪、祥雲紋樣、俐落墨線輪廓與和紙肌理，以平整套印色塊清晰框出數據與圖表。',
      '浮世絵木版画 | 藍色の大波や雲模様、力強い墨の輪郭線と和紙の風合いで伝統的な浮世絵に仕立てる。数値とグラフは平らな版画枠に明瞭に収める。',
      '우키요에 목판화 | 프러시안 블루 파도와 구름 문양, 굵은 먹선 윤곽과 화지 질감의 전통 목판화로 꾸미고 수치와 차트는 평탄한 판화 칸에 선명하게 담는다.',
      'Estampe ukiyo-e | Adoptez l’esthétique d’une gravure ukiyo-e : vagues bleu de Prusse, nuages stylisés, contours à l’encre et grain washi, avec panneaux nets pour les données.',
      'Ukiyo-e-Holzschnitt | Gestalte das Poster als japanischen Holzschnitt mit Preußischblau-Wellen, stilisierten Wolken, kräftigen Tuschekonturen und Washi-Struktur sowie klaren Datenfeldern.',
      'Grabado ukiyo-e | Diseña el póster como un grabado japonés ukiyo-e con olas azul de Prusia, nubes estilizadas, contornos en tinta y papel washi, con paneles limpios para los datos.'),
    popart: words(
      'Pop art poster | Build a bold mid-century pop-art graphic with saturated flat color blocks, thick black ink outlines, crisp Ben-Day halftone dots and comic-panel framing for each stat module.',
      '波普艺术 | 以高饱和平涂色块、粗黑漫画描边、精致 Ben-Day 网点与分格排版打造充满视觉冲击力的波普数据海报，每项看点独立成格。',
      '普普藝術 | 以高飽和平塗色塊、粗黑漫畫描邊、精緻 Ben-Day 網點與分格排版打造視覺衝擊力十足的普普數據海報，每項亮點獨立成格。',
      'ポップアート | 鮮やかなベタ面、太い黒の輪郭線、ハーフトーン網点、コミック風のコマ割りで力強く構成し、各指標を鮮明に見せる。',
      '팝아트 포스터 | 채도 높은 단색 블록, 굵은 검은 윤곽선, 벤데이 망점과 만화 패널 분할로 강렬한 팝아트 인포그래픽을 구성한다.',
      'Affiche pop art | Composez un visuel pop art percutant en aplats saturés, contours noirs épais, trames Ben-Day et cases façon bande dessinée pour chaque statistique.',
      'Pop-Art-Poster | Entwirf ein kraftvolles Pop-Art-Plakat mit satten Farbflächen, starken schwarzen Konturen, Ben-Day-Rasterpunkten und klaren Panel-Kästen.',
      'Póster pop art | Crea un cartel pop art llamativo con bloques de color saturado, contornos negros gruesos, puntos Ben-Day y viñetas nítidas para cada estadística.'),
    bauhaus: words(
      'Bauhaus geometric poster | Compose a crisp modernist Bauhaus poster using primary geometric shapes (circles, triangles, bars), strict asymmetric grid lines, bold geometric sans-serif numbers and flat matte color planes.',
      '包豪斯风格 | 运用经典包豪斯现代主义构成：圆形、三角与矩形几何色块、严谨不对称网格、粗犷几何无衬线数字与哑光平涂色面。',
      '包浩斯風格 | 運用經典包浩斯現代主義構成：圓形、三角與矩形幾何色塊、嚴謹不對稱格線、粗獷幾何無襯線數字與霧面平塗色面。',
      'バウハウス | 円・三角・矩形の基本幾何学形、厳格な非対称グリッド、太いジオメトリック・サンセリフ数字、マットな色面で端正に構成する。',
      '바우하우스 포스터 | 원·삼각형·직사각형의 기본 기하 도형, 엄격한 비대칭 그리드, 굵은 기하학적 산세리프 숫자와 평탄한 색면으로 구성한다.',
      'Affiche Bauhaus | Composez une affiche Bauhaus moderniste aux formes géométriques pures, grille asymétrique rigoureuse, chiffres sans empattement audacieux et aplats mats.',
      'Bauhaus-Plakat | Gestalte ein strenges Bauhaus-Plakat aus Kreisen, Dreiecken und Balken, asymmetrischem Raster, kräftigen geometrischen Ziffern und matten Farbflächen.',
      'Cartel Bauhaus | Compón un cartel Bauhaus modernista con formas geométricas puras, retícula asimétrica estricta, cifras geométricas audaces y planos de color mate.'),
    pixelart: words(
      'Retro pixel art HUD | Craft a crisp 16-bit retro arcade HUD poster with sharp pixel-grid borders, pixel-art keycaps and icons, stepped pixel bars and glowing high-contrast pixel typography on a rich dark screen.',
      '像素风 | 打造精致的 16-bit 复古街机/游戏机 HUD 面板：利落的像素阶梯边框、像素风键帽与小图标、锯齿阶梯柱状图以及深色屏幕上的高对比像素字。',
      '像素風 | 打造精緻的 16-bit 復古街機／遊戲機 HUD 面板：俐落的像素階梯邊框、像素風鍵帽與小圖示、階梯柱狀圖以及深色螢幕上的高對比像素字。',
      'レトロピクセルアート | 16bit アーケード HUD 風に、鋭いピクセル枠、ドット絵キーキャップやアイコン、階段状のドットグラフ、鮮やかなピクセル数字で仕上げる。',
      '레트로 픽셀 아트 | 16비트 아케이드 HUD 스타일로 선명한 픽셀 테두리, 도트 키캡·아이콘, 계단형 픽셀 차트와 고대비 픽셀 숫자를 구성한다.',
      'HUD pixel art rétro | Créez une affiche façon interface d’arcade 16 bits avec bordures en pixels nets, touches et icônes pixelisées, barres en escalier et chiffres contrastés.',
      'Retro-Pixel-Art-HUD | Gestalte ein scharfes 16-Bit-Arcade-HUD-Poster mit Pixelrahmen, Pixel-Tastenkappen und Icons, abgestuften Pixelbalken und kontrastreicher Pixelschrift.',
      'HUD pixel art retro | Diseña un póster estilo interfaz arcade de 16 bits con bordes pixelados nítidos, teclas e iconos en pixel art, barras escalonadas y tipografía de alto contraste.')
  };

  const chartWords = {
    columns: words(
      'Columns | Use evenly spaced time slots with equal-width columns and a shared zero baseline. Leave zero slots empty; label only notable counts.',
      '竖柱图 | 按时间顺序为每个给出的时段绘制一根等宽柱。所有柱共用零基线与同一比例尺。',
      '直條圖 | 依時間順序為每個提供的時段畫一根等寬直條。所有直條共用零基線與比例尺。',
      '縦棒 | 与えられた時間区分を順に等幅の縦棒で示す。全ての棒でゼロ基準と縮尺を揃える。',
      '세로 막대 | 주어진 시간 구간마다 같은 너비의 세로 막대를 시간순으로 그린다. 모든 막대에 같은 영점과 척도를 쓴다.',
      'Colonnes | Tracez une colonne de même largeur par intervalle fourni, en ordre chronologique. Utilisez une base zéro et une échelle communes.',
      'Säulen | Zeichne für jeden gelieferten Zeitabschnitt eine gleich breite Säule in zeitlicher Reihenfolge. Verwende überall dieselbe Skala mit gemeinsamer Nulllinie.',
      'Columnas | Dibuja una columna de igual ancho por intervalo dado, en orden temporal. Comparte la base cero y la escala.'),
    bars: words(
      'Horizontal bars | Put the earliest bin on top. Align the bars at zero, scale their lengths to counts and right-align the few values you label.',
      '横条图 | 最早的时段放在顶部。横条从共同零点出发，长度正比于计数，右侧用一列对齐精确值。',
      '橫條圖 | 最早的時段放在頂端。橫條自共同零點出發，長度正比於計數，右側成欄對齊確切數值。',
      '横棒 | 最も早い区分を上に置く。棒の出発点をゼロに揃え、長さで件数を示し、正確な値を右揃えで添える。',
      '가로 막대 | 가장 이른 시간 구간을 맨 위에 둔다. 같은 영점에서 시작해 횟수에 비례한 길이를 쓰고 정확한 값을 오른쪽에 맞춘다.',
      'Barres horizontales | Placez le premier intervalle en haut. Faites partir les barres de zéro, proportionnez-les aux comptes et alignez les valeurs exactes à droite.',
      'Waagerechte Balken | Beginne oben mit dem frühesten Zeitraum. Lasse alle Balken bei null beginnen, skaliere ihre Länge proportional und richte die exakten Werte rechts aus.',
      'Barras horizontales | Coloca arriba el primer intervalo. Parte de cero, usa longitudes proporcionales y alinea las cifras exactas a la derecha.'),
    clock: words(
      '24-hour dial | Place midnight at the top and space the hours equally clockwise. Draw a thin spoke for each observed hour, with length from the centre proportional to its count. Leave future or missing hours open.',
      '24 小时时钟 | 把零点放在正上方，顺时针等角排列各小时。每个已记录小时画一根从圆心向外的细辐条，长度正比于次数，未来或缺失时段留空。',
      '24 小時時鐘 | 將零點置於正上方，順時針等角排開各小時。每個已觀測小時從圓心向外畫一根細輻條，長度正比於次數。未來或缺漏時段留白。',
      '24 時間ダイヤル | 真上を午前 0 時とし、各時を時計回りに等角配置する。観測済みの時刻には中心から件数に比例する長さの細線を引く。未来と欠測の時間は空ける。',
      '24시간 다이얼 | 맨 위를 자정으로 두고 각 시각을 시계 방향으로 같은 각도에 배치한다. 관측된 시간은 중심에서 횟수에 비례하는 길이로 가는 살을 긋고 미래와 결측 구간은 비운다.',
      'Cadran de 24 heures | Placez minuit en haut et répartissez les heures à angles égaux dans le sens horaire. Pour chaque heure observée, tracez du centre un fin rayon de longueur proportionnelle au compte. Laissez vides les heures futures ou manquantes.',
      '24-Stunden-Zifferblatt | Setze Mitternacht nach oben und verteile die Stunden im Uhrzeigersinn in gleichen Winkeln. Zeichne für jede erfasste Stunde vom Zentrum eine dünne, zum Wert proportionale Speiche. Lasse künftige oder fehlende Stunden frei.',
      'Esfera de 24 horas | Coloca la medianoche arriba y reparte las horas en ángulos iguales en sentido horario. Traza desde el centro un radio fino de longitud proporcional al conteo de cada hora observada. Deja vacías las horas futuras o ausentes.'),
    dots: words(
      'Dot plot | Place equal-size dots for positive counts on a shared count axis, spaced by their actual times. Leave quiet intervals empty.',
      '点阵坐标图 | 每个时段用一个等大圆点，点位对应同一计数轴。各点独立呈现，时段标签按顺序排列。',
      '點位圖 | 每個時段放一個等大圓點，點的位置對應同一計數軸。各點獨立呈現，時段標示依序排列。',
      'ドットプロット | 各区分を一つの等大の点で共通の件数軸に載せる。点を独立させ、時間ラベルは順に並べる。',
      '점 도표 | 같은 개수 축에 구간마다 크기가 같은 점을 하나씩 놓는다. 점은 독립적으로 두고 시간 표기는 순서대로 배열한다.',
      'Diagramme à points | Placez un point de taille constante par intervalle sur une échelle commune des comptes. Laissez les points séparés et ordonnez les libellés dans le temps.',
      'Punktdiagramm | Setze pro Zeitabschnitt einen gleich großen Punkt auf eine gemeinsame Zählachse. Lasse die Punkte unverbunden und die Zeitbeschriftung chronologisch.',
      'Diagrama de puntos | Usa un punto del mismo tamaño por intervalo sobre una escala común de cantidades. Deja los puntos separados y los rótulos en orden temporal.'),
    heat: words(
      'Intensity strip | Give each time bin an equal cell. Use one light-to-dark scale with a key and label only the significant cells.',
      '色阶带 | 每个时段占等大一格，依时序排列。计数用同一条由浅至深的色阶表示，配色标，每格旁标出原数值。',
      '色階帶 | 每個時段各占等大一格，依時間順序排列。以同一組由淺至深的色階表示計數，附色標並為每格註明原數值。',
      '濃淡の帯 | 時間順に各区分を等大のマスへ置く。凡例付きの共通の明暗スケールで件数を表し、各マスに実数を添える。',
      '농도 띠 | 구간마다 같은 크기의 칸을 시간순으로 놓는다. 같은 명도 척도와 범례로 횟수를 표현하고 칸마다 실제 값을 붙인다.',
      'Bande d’intensité | Attribuez une case égale à chaque intervalle, dans l’ordre. Appliquez une échelle du clair au foncé avec légende et indiquez le compte exact de chaque case.',
      'Intensitätsband | Setze für jeden Zeitraum eine gleich große Zelle in zeitlicher Folge. Nutze eine gemeinsame Hell-Dunkel-Skala mit Legende und schreibe den exakten Wert an jede Zelle.',
      'Banda de intensidad | Asigna una celda igual a cada intervalo, en orden temporal. Aplica una sola escala de claro a oscuro con leyenda y anota el valor exacto de cada celda.'),
    line: words(
      'Measured line | Plot counts at their true times on one scale. Join observed values, including zeros at the baseline. Mark just the notable points and leave future intervals open.',
      '折线图 | 在同一坐标轴上逐点标出时段计数。相邻且已知的点以直线相连，缺失或未来时段保留空白。',
      '折線圖 | 以同一座標軸逐點標出時段計數。用直線連接相鄰且已知的點，缺漏或未來時段留白。',
      '折れ線 | 与えられた区分の件数を共通の軸上に点で示す。既知の隣り合う点を直線で結び、欠測や未来の区間は空ける。',
      '꺾은선 | 제공된 구간별 횟수를 같은 축에 점으로 표시한다. 인접한 관측점만 직선으로 잇고 결측 구간과 미래 구간은 비워 둔다.',
      'Courbe mesurée | Placez un point par mesure sur une échelle commune. Reliez par des segments droits les intervalles connus et contigus. Laissez vides les intervalles manquants ou futurs.',
      'Messkurve | Trage die gelieferten Werte als Punkte auf einer gemeinsamen Achse auf. Verbinde benachbarte bekannte Punkte geradlinig. Fehlende und zukünftige Abschnitte bleiben offen.',
      'Línea de mediciones | Marca los datos por intervalo en un eje común. Une con segmentos rectos solo los puntos conocidos contiguos. Deja huecos en los tramos ausentes o futuros.'),
    ledger: words(
      'Numeric ledger | Align the few nonzero time labels and counts that matter. An empty series needs only one brief note.',
      '数字清单 | 将给出的时段和计数对齐成简短清单，如实保留给出的零。若整个序列为空，只排一条空状态说明。',
      '數字清單 | 將提供的時段與計數對齊成簡短清單，保留給出的零值。若整個數列為空，只排一行無資料說明。',
      '数値リスト | 与えられた区分と件数を短い一覧に揃え、明示されたゼロは残す。系列そのものが空なら空状態の注記だけを置く。',
      '수치 목록 | 주어진 시간과 횟수를 짧은 목록으로 가지런히 정리하고 제공된 0은 그대로 쓴다. 계열 자체가 비어 있으면 빈 상태 안내만 둔다.',
      'Relevé chiffré | Alignez les intervalles et les comptes fournis dans une liste compacte. Conservez les zéros explicites. Une série vide se résume à une note signalant l’absence de valeurs.',
      'Zahlenliste | Ordne die gelieferten Zeiträume und Werte in einer kompakten Liste an. Übernimm vorgegebene Nullen. Bei einer leeren Reihe genügt ein Hinweis auf die fehlenden Einträge.',
      'Lista numérica | Alinea en una lista breve los intervalos y sus cantidades. Conserva los ceros facilitados. Si la serie está vacía, basta con la nota de ausencia de datos.')
  };

  const layoutWords = {
    masthead: words(
      'Masthead | Give the total and date a broad header. A fine rule separates it from the data below, with the supplied labels close to their values.',
      '报头式 | 顶部宽幅留给总数与日期，以一条细线与下方数据分隔。给出的标签紧邻对应数值。',
      '報頭式 | 頂端寬幅留給總數與日期，以一道細線與下方資料分隔。將提供的標籤貼近對應數值。',
      '見出し型 | 合計と日付を広い見出しに置く。細い罫線で下段のデータと区切り、与えられたラベルは値のそばに添える。',
      '제호형 | 넓은 머리말에 합계와 날짜를 둔다. 아래쪽 데이터와는 얇은 선으로 구분하고 제공된 표기는 값 가까이에 놓는다.',
      'Manchette | Réservez un large bandeau supérieur au total et à la date. Un filet le sépare des données en dessous. Placez les libellés fournis près de leurs valeurs.',
      'Breiter Kopf | Reserviere einen breiten Kopf für Summe und Datum. Eine feine Linie trennt ihn von den Daten darunter. Die vorgegebenen Beschriftungen stehen dicht bei ihren Werten.',
      'Cabecera | Reserva una cabecera amplia para el total y la fecha. Usa un filete para separarla de los datos inferiores y coloca cada rótulo dado cerca de su valor.'),
    split: words(
      'Offset columns | Give a narrow column to the date and summary, and a wider column to the chart. Align both to one top edge and preserve room for long numeric labels.',
      '错列双栏 | 以窄栏放日期和摘要，宽栏留给图形，两栏顶部对齐，为较长的数字标签预留空间。',
      '非對稱雙欄 | 以窄欄放置日期與摘要，寬欄留給圖形。兩欄頂端對齊，為較長的數字標示留足空間。',
      '非対称二段組み | 狭い列に日付と概要、広い列にグラフを置く。上端を揃え、長い数値ラベルに十分な幅を取る。',
      '비대칭 두 단 | 좁은 단에 날짜와 요약을, 넓은 단에 도표를 둔다. 위쪽을 맞추고 긴 숫자 표기를 위한 공간을 남긴다.',
      'Colonnes décalées | Réservez une colonne étroite à la date et au résumé, une colonne plus large au graphique. Alignez leurs bords supérieurs et ménagez de la place pour les nombres longs.',
      'Asymmetrische Spalten | Setze Datum und Zusammenfassung in eine schmale Spalte, die Grafik in eine breite. Richte die Oberkanten aus und plane Platz für lange Zahlen ein.',
      'Columnas asimétricas | Dedica una columna estrecha a la fecha y el resumen, y una más ancha al gráfico. Alinea sus bordes superiores y deja espacio para cifras largas.'),
    inset: words(
      'Inset plate | Center the total in an inset panel. Use a quiet frame to bind the chart and its labels beneath it, with the date at the upper edge.',
      '嵌框式 | 把总数置于嵌入式面板中央，日期排在上沿。下方用克制的边框把图形和标注围成一组。',
      '嵌框式 | 將總數置於嵌入面板中央，日期排在上緣。下方用細緻邊框將圖形與標示圍成一組。',
      '額装パネル | 内側のパネル中央に合計、上端に日付を置く。その下のグラフと注記は控えめな枠で束ねる。',
      '안쪽 액자 | 안쪽 패널 중앙에 합계를, 위쪽 가장자리에 날짜를 놓는다. 그 아래 차트와 표기를 얇은 테두리로 묶는다.',
      'Plaque encadrée | Centrez le total dans un panneau en retrait, avec la date sur son bord supérieur. Réunissez sous celui-ci le graphique et ses légendes dans un cadre discret.',
      'Eingefasste Tafel | Zentriere die Summe in einem eingelassenen Feld mit Datum am oberen Rand. Fasse darunter Grafik und Beschriftung mit einem dezenten Rahmen zusammen.',
      'Panel enmarcado | Centra el total en un panel interior y coloca la fecha en el borde superior. Agrupa debajo el gráfico y sus rótulos con un marco discreto.'),
    bands: words(
      'Horizontal bands | Stack the supplied content in left-aligned horizontal bands. Group each value with its label and give the main data display the most height.',
      '横带式 | 把已有内容分成靠左对齐的横带。每个数值与标签放在一起，主数据区占用最多高度。',
      '橫帶式 | 將已有內容分成靠左對齊的橫帶。將每個數值與其標籤排在一起，主要資料區留最多高度。',
      '横帯構成 | 与えられた内容を左揃えの横帯に分ける。値とラベルを一組にし、主要なデータの帯に最も高さを与える。',
      '가로 띠 | 주어진 내용을 왼쪽에 맞춘 가로 띠로 나눈다. 수치와 표기를 한데 묶고 주요 데이터를 보여 주는 띠에 가장 큰 높이를 준다.',
      'Bandes horizontales | Répartissez le contenu fourni en bandes horizontales alignées à gauche. Gardez chaque valeur près de son libellé et réservez le plus de hauteur aux données principales.',
      'Querbänder | Ordne die vorgegebenen Inhalte in linksbündigen Querbändern an. Belasse Werte bei ihren Beschriftungen und gib der zentralen Datendarstellung das höchste Band.',
      'Franjas horizontales | Distribuye el contenido dado en franjas horizontales alineadas a la izquierda. Agrupa cada valor con su rótulo y reserva la franja más alta para los datos principales.'),
    bento: words(
      'Editorial bento grid | Top hero banner with the oversized total and keyboard-vs-mouse split bar, middle focal chart panel, and bottom modular stat cards for shortcuts, peak activity and book comparison.',
      '便当盒卡片网格 | 顶部大号总数与键鼠占比横条，中部独立主图表区，底部模块化数据卡片分别呈现快捷键键帽、峰值时段与文学字数对比。',
      '便當盒卡片網格 | 頂部大號總數與鍵鼠佔比橫條，中部獨立主圖表區，底部模組化數據卡片分別呈現快捷鍵鍵帽、尖峰時段與文學字數對比。',
      'ベントグリッド | 上部に大きな合計とキーボード／マウス比率バー、中央に主グラフ、下部にショートカット・ピーク・読書比較のモジュールカードを整理して配置する。',
      '벤토 그리드 | 상단에 대형 합계와 키보드·마우스 비율 바, 중앙에 핵심 차트, 하단에 단축키·피크·도서 비교 모듈 카드를 정돈해 배치한다.',
      'Grille bento éditoriale | Bandeau supérieur avec grand total et barre clavier/souris, graphique central, puis cartes modulaires en bas pour raccourcis, pic et comparaison littéraire.',
      'Bento-Kartenraster | Oben großer Gesamtwert mit Tastatur/Maus-Leiste, in der Mitte das Hauptdiagramm, unten modulare Stat-Karten für Shortcuts, Spitzenstunde und Buchvergleich.',
      'Cuadrícula bento | Cabecera con gran total y barra teclado/ratón, gráfico principal en el centro y tarjetas modulares abajo para atajos, pico y comparación literaria.')
  };

  const palettes = {
    indigo:   [['#F0F3FA', '#162749', '#304FAF', '#687689'], ['Porcelain & indigo','瓷白靛蓝','瓷白靛藍','磁器と藍','백자와 쪽빛','Porcelaine et indigo','Porzellan und Indigo','Porcelana e índigo']],
    vermilion:[['#FFF4E9', '#351F23', '#B83F27', '#756157'], ['Vermilion on ivory','象牙朱红','象牙朱紅','象牙と朱','상아와 주홍','Vermillon sur ivoire','Zinnober auf Elfenbein','Bermellón sobre marfil']],
    teal:     [['#EBF8F3', '#163C3C', '#006D71', '#87664B'], ['Sea glass','海玻璃','海玻璃','シーグラス','바다 유리','Verre de mer','Meerglas','Vidrio marino']],
    moss:     [['#F2F2DB', '#303923', '#4F672E', '#935535'], ['Moss & clay','苔绿陶土','苔綠陶土','苔と土','이끼와 점토','Mousse et argile','Moos und Lehm','Musgo y arcilla']],
    plum:     [['#F9EDF1', '#382438', '#803D71', '#6B5863'], ['Plum & rose','梅紫蔷薇','梅紫薔薇','プラムと薔薇','자두와 장미','Prune et rose','Pflaume und Rosé','Ciruela y rosa']],
    gold:     [['#2E291D', '#FCF4DA', '#F8C64F', '#CEC3A5'], ['Brass at dusk','暮色黄铜','暮色黃銅','黄昏の真鍮','황혼의 황동','Laiton au crépuscule','Messing im Zwielicht','Latón al anochecer']],
    graphite: [['#F3F2EE', '#24282B', '#3F525D', '#737270'], ['Graphite & chalk','石墨粉笔','石墨粉筆','黒鉛とチョーク','흑연과 분필','Graphite et craie','Graphit und Kreide','Grafito y tiza']],
    night:    [['#182937', '#EFF7F6', '#77D6CE', '#B0BDD0'], ['Midnight lagoon','午夜泻湖','午夜潟湖','真夜中のラグーン','한밤의 석호','Lagon de minuit','Mitternachtslagune','Laguna de medianoche']]
  };
  const paletteRoles = text([
    ['Background','Text','Main accent','Secondary marks'],
    ['底色','文字','主强调色','辅助标记'],
    ['底色','文字','主強調色','輔助標記'],
    ['背景','文字','主アクセント','補助マーク'],
    ['배경','글자','주 강조색','보조 표식'],
    ['Fond','Texte','Accent principal','Repères secondaires'],
    ['Grund','Schrift','Hauptakzent','Nebenmarken'],
    ['Fondo','Texto','Acento principal','Marcas secundarias']
  ]);

  // Include only pairings that keep category colors and quantitative marks legible
  // within the medium. The quiet ledger works with every medium for sparse data.
  // The budget caps optional facts. Wide editorial grids can carry more detail
  // than a display or sleeve where the main figure needs room.
  const mediaOptions = {
    risograph:    [3, ['columns','dots','heat'], ['indigo','vermilion','teal','plum'], ['split','masthead','bands']],
    letterpress:  [3, ['bars','clock','line'], ['indigo','vermilion','moss','graphite'], ['masthead','inset','bands']],
    transit:      [3, ['line','dots','bars'], ['indigo','teal','gold','night'], ['bands','split','masthead']],
    swiss:        [2, ['columns','dots','bars','heat'], ['vermilion','indigo','graphite','plum'], ['split','bands','masthead']],
    blueprint:    [4, ['line','clock','dots'], ['indigo','night','teal'], ['split','inset','masthead']],
    woodcut:      [2, ['bars','columns','clock'], ['moss','vermilion','graphite','gold'], ['inset','masthead','bands']],
    eink:         [2, ['dots','line','clock'], ['graphite','moss','indigo'], ['split','inset','bands']],
    textile:      [3, ['heat','bars','columns'], ['moss','plum','indigo','gold'], ['inset','bands','split']],
    glass:        [4, ['columns','bars','clock'], ['night','gold','indigo'], ['bento','inset','bands']],
    contact:      [4, ['heat','dots','columns'], ['graphite','night','plum','gold'], ['bands','split','inset']],
    papercut:     [3, ['columns','heat','bars'], ['plum','vermilion','teal','moss'], ['split','inset','bands']],
    lcd:          [2, ['line','dots','clock'], ['moss','gold','graphite','night'], ['inset','masthead','bands']],
    keycaps:      [3, ['heat','columns','dots'], ['teal','indigo','plum','vermilion'], ['bento','inset','bands','split']],
    terminal:     [4, ['bars','line','heat'], ['night','gold','graphite','teal'], ['bands','split','masthead']],
    newsprint:    [4, ['bars','clock','line','columns'], ['graphite','indigo','vermilion','moss'], ['masthead','bands','split']],
    album:        [2, ['dots','clock','heat'], ['plum','night','vermilion','gold'], ['inset','split','masthead']],
    inkwash:      [4, ['columns','bars','line'], ['graphite','vermilion','moss','indigo'], ['masthead','bands','split']],
    ukiyoe:       [4, ['columns','bars','clock'], ['indigo','vermilion','gold','teal'], ['inset','masthead','bands']],
    popart:       [4, ['columns','bars','dots'], ['vermilion','indigo','plum','gold'], ['bento','split','bands']],
    bauhaus:      [4, ['columns','bars','dots'], ['vermilion','indigo','graphite','gold'], ['bento','split','masthead']],
    pixelart:     [4, ['columns','bars','heat'], ['night','gold','teal','plum'], ['bento','inset','bands']]
  };
  const medium = {};
  for (const [id, [budget, charts, palettes, layouts]] of Object.entries(mediaOptions)) {
    medium[id] = Object.freeze({
      budget, words: mediumWords[id], charts: Object.freeze([...charts, 'ledger']),
      palettes: Object.freeze(palettes), layouts: Object.freeze(layouts)
    });
  }
  const palette = {};
  for (const [id, [colors, labels]] of Object.entries(palettes)) {
    palette[id] = Object.freeze(Object.fromEntries(locales.map((locale, index) => [
      locale, Object.freeze({
        label: labels[index],
        instruction: paletteRoles[locale].map((role, i) => role + ' ' + colors[i]).join(' · ')
      })
    ])));
  }

  const registry = Object.freeze({
    medium: Object.freeze(medium), chart: Object.freeze(chartWords),
    palette: Object.freeze(palette), layout: Object.freeze(layoutWords)
  });
  const mediumIds = Object.keys(medium);
  const dimensions = ['medium','chart','palette','layout'];

  function localeFor(locale) {
    if (locales.includes(locale)) return locale;
    const code = String(locale || 'en').toLowerCase().replaceAll('_', '-');
    if (code.startsWith('zh')) return /-(tw|hk|mo|hant)(-|$)/.test(code) ? 'zh-TW' : 'zh-CN';
    const base = code.split('-')[0];
    return locales.includes(base) ? base : 'en';
  }

  function chartChoices(m, context) {
    // Daily series may hold many measured hourly bins within ONE observed day.
    // Prefer an explicit density computed from the exact series being plotted.
    const sparse = context.sparse ?? (context.total === 0 || context.nonzeroBuckets < 2 ||
      (context.kind === 'week' && context.observedDays < 2));
    return sparse ? ['ledger'] : m.charts.filter(chart => chart !== 'ledger' &&
      (chart !== 'clock' || context.kind === 'day'));
  }

  function choices(m, context) {
    return {chart: chartChoices(m, context), palette: m.palettes, layout: m.layouts};
  }

  function isCompatible(recipe, context = {}) {
    if (!recipe || !Object.prototype.hasOwnProperty.call(medium, recipe.medium)) return false;
    return Object.entries(choices(medium[recipe.medium], context))
      .every(([dimension, allowed]) => allowed.includes(recipe[dimension]));
  }

  function describe(recipe, locale = 'en') {
    const lang = localeFor(locale);
    return Object.fromEntries(dimensions.map(dimension => {
      const id = recipe[dimension];
      const variants = dimension === 'medium' ? medium[id].words : registry[dimension][id];
      return [dimension, {id, ...variants[lang]}];
    }));
  }

  function createPicker(random = Math.random) {
    let active = null;
    let unused = new Set(mediumIds);

    function choose(items, previous) {
      const alternatives = items.filter(item => item !== previous);
      const pool = alternatives.length ? alternatives : items;
      return pool[Math.floor(random() * pool.length)];
    }

    function media(context, preserveOthers) {
      const pool = unused.size ? [...unused] : mediumIds;
      const candidates = list => list.filter(id => id !== active?.medium &&
        (!preserveOthers || isCompatible({...active, medium: id}, context)));
      const fresh = candidates(pool);
      // A deliberate medium-only edit should still work when the remaining bag
      // has no match for the selected chart, colors and layout.
      return preserveOthers && !fresh.length ? candidates(mediumIds) : fresh;
    }

    function fullRecipe(context) {
      const id = choose(media(context, false));
      const allowed = choices(medium[id], context);
      if (!unused.size) unused = new Set(mediumIds);
      unused.delete(id);
      return Object.freeze({
        medium: id, chart: choose(allowed.chart, active?.chart),
        palette: choose(allowed.palette, active?.palette), layout: choose(allowed.layout, active?.layout)
      });
    }

    function adapt(context = {}) {
      if (!active) active = fullRecipe(context);
      else if (!isCompatible(active, context)) {
        const charts = chartChoices(medium[active.medium], context);
        active = Object.freeze({...active, chart:choose(charts, active.chart)});
      }
      return active;
    }

    function canChange(context = {}, dimension = 'all') {
      if (dimension === 'all') return true;
      if (!dimensions.includes(dimension)) return false;
      if (!active) return dimension !== 'chart' || chartChoices(medium[mediumIds[0]], context).length > 1;
      if (dimension === 'medium') return media(context, true).length > 0;
      const allowed = choices(medium[active.medium], context)[dimension];
      return !!allowed?.some(id => id !== active[dimension]);
    }

    function next(context = {}, dimension = 'all') {
      if (!active || dimension === 'all') {
        active = fullRecipe(context);
        return active;
      }
      adapt(context);
      if (!canChange(context, dimension)) return active;
      let id;
      if (dimension === 'medium') {
        id = choose(media(context, true));
        if (!unused.size) unused = new Set(mediumIds);
        unused.delete(id);
      } else {
        id = choose(choices(medium[active.medium], context)[dimension], active[dimension]);
      }
      active = Object.freeze({...active, [dimension]:id});
      return active;
    }

    return {next, current: () => active, adapt, canChange};
  }

  return Object.freeze({registry, createPicker, describe, isCompatible});
})();
if (typeof window !== 'undefined') window.ClickPulseReportRecipes = ClickPulseReportRecipes;
if (typeof module !== 'undefined') module.exports = ClickPulseReportRecipes;
