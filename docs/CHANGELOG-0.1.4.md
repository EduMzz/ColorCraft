# ColorCraft 0.1.4

Adiciona o painel **Fontes** e captura de cores a partir de PNG, JPEG, WebP, TIFF, PDF e SVG estático. Cada fonte pode permanecer vinculada ou ser incorporada ao projeto. Capturas usam pixel exato, média/mediana de área ou máscara pintada; cores vinculadas são bloqueadas e atualizadas junto com a fonte em uma transação única.

O formato `.colorcraft` passa para schema v4. Projetos v3 são migrados ao abrir sem mudar cores, metadados, relações ou histórico. O projeto salva caminhos, fingerprint, miniatura, métodos, posições e máscaras; a cópia incorporada tem limite de 64 MiB por fonte e 180 MiB totais. Atualizações de arquivo exigem confirmação antes de recalcular valores. Perfil ICC não suportado interrompe a captura com uma explicação.

O pacote Windows inclui PDFium 7881 e os avisos/licenças do binário e das bibliotecas distribuídas junto dele. A 0.1.3 e suas distribuições permanecem preservadas.
