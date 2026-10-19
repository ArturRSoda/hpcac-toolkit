# Configuração latexmk para o projeto lapesd-thesis
# Roda makeglossaries automaticamente quando os arquivos .acn/.glo mudam.
# -shell-escape necessário para a extração do logo da UFSC (embeddedlogo).
$pdflatex = 'pdflatex -shell-escape %O %S';

add_cus_dep('glo', 'gls', 0, 'run_makeglossaries');
add_cus_dep('acn', 'acr', 0, 'run_makeglossaries');

sub run_makeglossaries {
    system("makeglossaries '$_[0]'");
}

push @generated_exts, 'glo', 'gls', 'glg';
push @generated_exts, 'acn', 'acr', 'alg';
