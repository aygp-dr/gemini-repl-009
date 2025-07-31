;;; gemini-repl-009.el --- Emacs configuration for Gemini REPL project -*- lexical-binding: t; -*-

;; Author: Your Name
;; Keywords: gemini, repl, org-mode, python
;; Package-Requires: ((emacs "28.1") (org "9.5") (python-mode "6.3"))

;;; Commentary:
;; Configuration for working with the Gemini REPL project
;; Supports org-mode with Babel, Python development, and Mermaid diagrams

;;; Code:

(require 'org)
(require 'ob-python)
(require 'ob-shell)
(require 'ob-mermaid)

;; Project-specific variables
(defgroup gemini-repl nil
  "Gemini REPL project configuration."
  :group 'programming)

(defcustom gemini-repl-project-root
  (expand-file-name "~/projects/aygp-dr/gemini-repl-009/")
  "Root directory of the Gemini REPL project."
  :type 'directory
  :group 'gemini-repl)

(defcustom gemini-repl-python-executable
  (executable-find "python3")
  "Python executable to use for the project."
  :type 'string
  :group 'gemini-repl)

;; Org-mode Babel configuration
(org-babel-do-load-languages
 'org-babel-load-languages
 '((python . t)
   (shell . t)
   (mermaid . t)
   (emacs-lisp . t)))

;; Set up Python environment for org-babel
(setq org-babel-python-command gemini-repl-python-executable)

;; Enable :mkdirp for all source blocks
(setq org-babel-default-header-args
      '((:mkdirp . "yes")
        (:results . "output")
        (:exports . "both")))

;; Specific header args for Python blocks with tangle support
(setq org-babel-default-header-args:python
      '((:results . "output")
        (:exports . "both")
        (:mkdirp . "yes")
        (:tangle . "yes")))

;; Mermaid configuration
(setq ob-mermaid-cli-path "/usr/local/bin/mmdc")  ; Adjust path as needed

;; Function to set up project-specific environment
(defun gemini-repl-setup-environment ()
  "Set up environment variables for Gemini REPL project."
  (interactive)
  (let ((env-file (expand-file-name ".env" gemini-repl-project-root)))
    (when (file-exists-p env-file)
      (with-temp-buffer
        (insert-file-contents env-file)
        (goto-char (point-min))
        (while (re-search-forward "^\\([^#=]+\\)=\\(.+\\)$" nil t)
          (setenv (match-string 1) (match-string 2)))))))

;; Function to create a new experiment org file
(defun gemini-repl-new-experiment (name)
  "Create a new experiment org file with NAME."
  (interactive "sExperiment name: ")
  (let* ((filename (format "%s.org" (replace-regexp-in-string " " "-" name)))
         (filepath (expand-file-name filename 
                                    (expand-file-name "experiments" gemini-repl-project-root))))
    (find-file filepath)
    (insert (format "#+TITLE: %s
#+AUTHOR: %s
#+DATE: %s
#+PROPERTY: header-args :mkdirp yes

* Overview

* Setup
#+begin_src python :tangle setup.py
import os
import sys
sys.path.insert(0, '%s/src')
#+end_src

* Implementation

* Testing

* Results

* Diagrams
#+begin_src mermaid :file diagrams/architecture.png
graph TD
    A[Client] --> B[API Gateway]
    B --> C[Gemini Service]
    C --> D[Model]
#+end_src
" name user-full-name (format-time-string "%Y-%m-%d") gemini-repl-project-root))))

;; Function to run tests
(defun gemini-repl-run-tests ()
  "Run the project test suite."
  (interactive)
  (let ((default-directory gemini-repl-project-root))
    (compile "./run_baseline_tests.sh")))

;; Function to test API
(defun gemini-repl-test-api ()
  "Test the API endpoints."
  (interactive)
  (let ((default-directory gemini-repl-project-root))
    (compile "./test_api_live.sh")))

;; Function to open project README
(defun gemini-repl-open-readme ()
  "Open the project README.org file."
  (interactive)
  (find-file (expand-file-name "README.org" gemini-repl-project-root)))

;; Function to tangle all org files in experiments directory
(defun gemini-repl-tangle-experiments ()
  "Tangle all org files in the experiments directory."
  (interactive)
  (let ((experiments-dir (expand-file-name "experiments" gemini-repl-project-root)))
    (dolist (file (directory-files experiments-dir t "\\.org$"))
      (with-current-buffer (find-file-noselect file)
        (org-babel-tangle)
        (message "Tangled %s" file)))))

;; Custom keybindings for the project
(defvar gemini-repl-mode-map
  (let ((map (make-sparse-keymap)))
    (define-key map (kbd "C-c g e") 'gemini-repl-setup-environment)
    (define-key map (kbd "C-c g n") 'gemini-repl-new-experiment)
    (define-key map (kbd "C-c g t") 'gemini-repl-run-tests)
    (define-key map (kbd "C-c g a") 'gemini-repl-test-api)
    (define-key map (kbd "C-c g r") 'gemini-repl-open-readme)
    (define-key map (kbd "C-c g T") 'gemini-repl-tangle-experiments)
    map)
  "Keymap for Gemini REPL mode.")

;; Minor mode for the project
(define-minor-mode gemini-repl-mode
  "Minor mode for Gemini REPL project development."
  :lighter " GeminiREPL"
  :keymap gemini-repl-mode-map
  (when gemini-repl-mode
    (gemini-repl-setup-environment)))

;; Auto-enable the mode for project files
(defun gemini-repl-maybe-enable ()
  "Enable gemini-repl-mode if in project directory."
  (when (and buffer-file-name
             (string-prefix-p (expand-file-name gemini-repl-project-root)
                            (expand-file-name buffer-file-name)))
    (gemini-repl-mode 1)))

(add-hook 'find-file-hook 'gemini-repl-maybe-enable)
(add-hook 'org-mode-hook 'gemini-repl-maybe-enable)
(add-hook 'python-mode-hook 'gemini-repl-maybe-enable)

;; Directory local variables for the project
(dir-locals-set-class-variables
 'gemini-repl-project
 `((nil . ((eval . (gemini-repl-mode 1))))
   (org-mode . ((org-confirm-babel-evaluate . nil)
                (org-src-preserve-indentation . t)))
   (python-mode . ((python-shell-interpreter . ,gemini-repl-python-executable)
                   (python-shell-interpreter-args . "-i")))))

(dir-locals-set-directory-class
 gemini-repl-project-root
 'gemini-repl-project)

(provide 'gemini-repl-009)
;;; gemini-repl-009.el ends here
