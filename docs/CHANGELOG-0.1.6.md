# ColorCraft 0.1.6

- Janela de captura reorganizada com ferramentas vetoriais verticais à esquerda, painel de imagem, painéis separados para lista e opções, e rodapé fixo com Aplicar capturas e Cancelar.
- Mão para deslocar livremente a imagem e lupa com ampliação no ponto clicado, redução por Alt/botão direito e roda do mouse; Ajustar centraliza novamente.
- Títulos longos são truncados, a lista e o inspetor rolam separadamente e os controles permanecem acessíveis em janela reduzida.
- Cliques em marcadores selecionam a captura existente; arrastar usa o ponto de início do gesto e mantém o ID, sem criar duplicatas. Marcadores numerados, com lente durante o arraste.
- Pintar sobre uma captura pontual cria outra captura de máscara, preservando a primeira. Traços seguintes continuam apenas a máscara selecionada na mesma página. Mover o marcador de uma máscara desloca também seus traços.
- Com o pincel ativo, o botão **Nova máscara** inicia outra captura independente. O valor é recalculado ao terminar o traço; apenas navegar, selecionar ou ampliar a prévia não recalcula capturas salvas.
- PDF com indicação ICCBased oferece usar explicitamente os pixels renderizados pelo PDFium ou cancelar para converter uma cópia em outro aplicativo. Atualização e relincagem também exigem a escolha antes de recalcular cores.
- Pasta e ZIP novos, preservando distribuições e instâncias anteriores.
